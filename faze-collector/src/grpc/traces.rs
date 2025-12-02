use crate::{
    convert::traces::convert_resource_spans,
    proto::opentelemetry::proto::collector::trace::v1::{
        ExportTracePartialSuccess, ExportTraceServiceRequest, ExportTraceServiceResponse,
        trace_service_server::{TraceService, TraceServiceServer},
    },
};
use faze::Storage;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::error;

/// OTLP collector that receives traces via gRPC
pub struct OtlpSpansCollector {
    storage: Arc<Storage>,
}

impl OtlpSpansCollector {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    pub fn into_service(self) -> TraceServiceServer<Self> {
        TraceServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl TraceService for OtlpSpansCollector {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let req = request.into_inner();
        let spans = convert_resource_spans(&req.resource_spans);
        let mut rejected_spans = 0;
        let mut error_messages = Vec::new();

        for span in &spans {
            if let Err(e) = self.storage.insert_span(span) {
                error!("Failed to insert span {}: {}", span.span_id, e);
                rejected_spans += 1;
                error_messages.push(format!("span {}: {}", span.span_id, e));
            }
        }

        let response = if rejected_spans > 0 {
            ExportTraceServiceResponse {
                partial_success: Some(ExportTracePartialSuccess {
                    rejected_spans,
                    error_message: error_messages.join("; "),
                }),
            }
        } else {
            ExportTraceServiceResponse {
                partial_success: None,
            }
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::proto::{
        common::v1::{AnyValue, KeyValue, any_value},
        resource::v1::Resource,
        trace::v1::{
            ResourceSpans, ScopeSpans, Span as OtlpSpan, SpanKind as OtlpSpanKind,
            Status as OtlpStatus, StatusCode as OtlpStatusCode,
        },
    };

    fn create_test_otlp_span(trace_id: &[u8], span_id: &[u8], name: &str) -> OtlpSpan {
        OtlpSpan {
            trace_id: trace_id.to_vec(),
            span_id: span_id.to_vec(),
            trace_state: String::new(),
            parent_span_id: vec![],
            name: name.to_string(),
            kind: OtlpSpanKind::Server as i32,
            start_time_unix_nano: 1_000_000_000_000_000_000,
            end_time_unix_nano: 1_000_000_000_100_000_000,
            attributes: vec![KeyValue {
                key: "http.method".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("GET".to_string())),
                }),
            }],
            dropped_attributes_count: 0,
            events: vec![],
            dropped_events_count: 0,
            links: vec![],
            dropped_links_count: 0,
            status: Some(OtlpStatus {
                message: String::new(),
                code: OtlpStatusCode::Ok as i32,
            }),
        }
    }

    #[tokio::test]
    async fn test_export_traces() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![create_test_otlp_span(
                        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        &[1, 2, 3, 4, 5, 6, 7, 8],
                        "GET /api/users",
                    )],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());

        let count = storage.count_spans().unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_export_multiple_spans() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![
                        create_test_otlp_span(
                            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            &[1, 2, 3, 4, 5, 6, 7, 8],
                            "span1",
                        ),
                        create_test_otlp_span(
                            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            &[9, 10, 11, 12, 13, 14, 15, 16],
                            "span2",
                        ),
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());

        let count = storage.count_spans().unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_export_empty_request() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let request = ExportTraceServiceRequest {
            resource_spans: vec![],
        };

        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_spans().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_export_multiple_resource_spans() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let request = ExportTraceServiceRequest {
            resource_spans: vec![
                ResourceSpans {
                    resource: Some(Resource {
                        attributes: vec![KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("service-1".to_string())),
                            }),
                        }],
                        dropped_attributes_count: 0,
                    }),
                    scope_spans: vec![ScopeSpans {
                        scope: None,
                        spans: vec![create_test_otlp_span(
                            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            &[1, 2, 3, 4, 5, 6, 7, 8],
                            "service-1-span",
                        )],
                        schema_url: String::new(),
                    }],
                    schema_url: String::new(),
                },
                ResourceSpans {
                    resource: Some(Resource {
                        attributes: vec![KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("service-2".to_string())),
                            }),
                        }],
                        dropped_attributes_count: 0,
                    }),
                    scope_spans: vec![ScopeSpans {
                        scope: None,
                        spans: vec![create_test_otlp_span(
                            &[2, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            &[2, 2, 3, 4, 5, 6, 7, 8],
                            "service-2-span",
                        )],
                        schema_url: String::new(),
                    }],
                    schema_url: String::new(),
                },
            ],
        };

        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_spans().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_export_span_with_error_status() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let mut span = create_test_otlp_span(
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            &[1, 2, 3, 4, 5, 6, 7, 8],
            "error-span",
        );
        span.status = Some(OtlpStatus {
            code: OtlpStatusCode::Error as i32,
            message: "Internal server error".to_string(),
        });

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("error-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![span],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());

        let trace = storage
            .get_trace_by_id("0102030405060708090a0b0c0d0e0f10")
            .unwrap();
        assert!(trace.has_errors());
    }

    #[tokio::test]
    async fn test_export_span_with_all_kinds() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let kinds = [
            OtlpSpanKind::Unspecified,
            OtlpSpanKind::Internal,
            OtlpSpanKind::Server,
            OtlpSpanKind::Client,
            OtlpSpanKind::Producer,
            OtlpSpanKind::Consumer,
        ];

        let mut spans = Vec::new();
        for (i, kind) in kinds.iter().enumerate() {
            let mut span = create_test_otlp_span(
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                &[i as u8 + 1, 2, 3, 4, 5, 6, 7, 8],
                &format!("span-{}", i),
            );
            span.kind = *kind as i32;
            spans.push(span);
        }

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_spans().unwrap(), 6);
    }

    #[tokio::test]
    async fn test_export_span_with_parent() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let parent_span = create_test_otlp_span(
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            &[1, 2, 3, 4, 5, 6, 7, 8],
            "parent-span",
        );

        let mut child_span = create_test_otlp_span(
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            &[9, 10, 11, 12, 13, 14, 15, 16],
            "child-span",
        );
        child_span.parent_span_id = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![parent_span, child_span],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());

        let trace = storage
            .get_trace_by_id("0102030405060708090a0b0c0d0e0f10")
            .unwrap();
        assert_eq!(trace.spans.len(), 2);

        let child = trace
            .spans
            .iter()
            .find(|s| s.span_id == "090a0b0c0d0e0f10")
            .unwrap();
        assert_eq!(child.parent_span_id, Some("0102030405060708".to_string()));
    }

    #[tokio::test]
    async fn test_export_span_with_complex_attributes() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let mut span = create_test_otlp_span(
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            &[1, 2, 3, 4, 5, 6, 7, 8],
            "complex-span",
        );

        span.attributes = vec![
            KeyValue {
                key: "http.method".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("POST".to_string())),
                }),
            },
            KeyValue {
                key: "http.status_code".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::IntValue(201)),
                }),
            },
            KeyValue {
                key: "http.response_time".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::DoubleValue(0.123)),
                }),
            },
            KeyValue {
                key: "cache.hit".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::BoolValue(true)),
                }),
            },
        ];

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![span],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());

        let trace = storage
            .get_trace_by_id("0102030405060708090a0b0c0d0e0f10")
            .unwrap();
        let span = &trace.spans[0];

        assert_eq!(span.attributes.get_string("http.method"), Some("POST"));
        assert_eq!(span.attributes.get_int("http.status_code"), Some(201));
        assert_eq!(
            span.attributes.get_double("http.response_time"),
            Some(0.123)
        );
        assert_eq!(span.attributes.get_bool("cache.hit"), Some(true));
    }

    #[tokio::test]
    async fn test_export_traces_sequential() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpSpansCollector::new(storage.clone());

        let request1 = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![create_test_otlp_span(
                        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        &[1, 2, 3, 4, 5, 6, 7, 8],
                        "test-span-1",
                    )],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let response1 = collector
            .export(Request::new(request1))
            .await
            .unwrap()
            .into_inner();
        assert!(response1.partial_success.is_none());
        assert_eq!(storage.count_spans().unwrap(), 1);

        let request2 = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![create_test_otlp_span(
                        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        &[2, 2, 3, 4, 5, 6, 7, 8],
                        "test-span-2",
                    )],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let response2 = collector
            .export(Request::new(request2))
            .await
            .unwrap()
            .into_inner();
        assert!(response2.partial_success.is_none());
        assert_eq!(storage.count_spans().unwrap(), 2);
    }
}
