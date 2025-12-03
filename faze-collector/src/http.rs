use crate::{
    convert::traces::convert_resource_spans,
    proto::opentelemetry::proto::collector::trace::v1::ExportTraceServiceRequest,
};
use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use faze::Storage;
use prost::Message;
use std::sync::Arc;
use tracing::error;

/// HTTP handler for OTLP trace export
async fn export_traces(
    State(storage): State<Arc<Storage>>,
    body: Bytes,
) -> Result<Response, StatusCode> {
    let request = ExportTraceServiceRequest::decode(body).map_err(|e| {
        error!("Failed to decode protobuf: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let spans = convert_resource_spans(&request.resource_spans);
    let mut rejected_spans = 0;
    let mut error_messages = Vec::new();

    for span in &spans {
        if let Err(e) = storage.insert_span(span) {
            error!("Failed to insert span {}: {}", span.span_id, e);
            rejected_spans += 1;
            error_messages.push(format!("span {}: {}", span.span_id, e));
        }
    }

    if rejected_spans > 0 {
        error!(
            "Rejected {} spans: {}",
            rejected_spans,
            error_messages.join("; ")
        );
        Ok((StatusCode::INTERNAL_SERVER_ERROR, error_messages.join("; ")).into_response())
    } else {
        Ok(StatusCode::OK.into_response())
    }
}

/// Create HTTP router for OTLP collector
pub fn create_router(storage: Arc<Storage>) -> Router {
    Router::new()
        .route("/v1/traces", post(export_traces))
        .with_state(storage)
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn create_test_request() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
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
                    spans: vec![OtlpSpan {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        trace_state: String::new(),
                        parent_span_id: vec![],
                        name: "test-span".to_string(),
                        kind: OtlpSpanKind::Server as i32,
                        start_time_unix_nano: 1_000_000_000_000_000_000,
                        end_time_unix_nano: 1_000_000_000_100_000_000,
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: Some(OtlpStatus {
                            message: String::new(),
                            code: OtlpStatusCode::Ok as i32,
                        }),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    #[tokio::test]
    async fn test_export_traces_http() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let request_data = create_test_request();
        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();
        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/x-protobuf")
            .body(Body::from(buf))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let count = storage.count_spans().unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_export_traces_invalid_protobuf() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let invalid_data = b"not a valid protobuf message";
        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/x-protobuf")
            .body(Body::from(&invalid_data[..]))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_export_traces_empty_body() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let empty_request = ExportTraceServiceRequest {
            resource_spans: vec![],
        };
        let mut buf = Vec::new();
        empty_request.encode(&mut buf).unwrap();
        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/x-protobuf")
            .body(Body::from(buf))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.count_spans().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_export_traces_multiple_spans() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());

        let request_data = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(
                                "multi-span-service".to_string(),
                            )),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![
                        OtlpSpan {
                            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                            trace_state: String::new(),
                            parent_span_id: vec![],
                            name: "span1".to_string(),
                            kind: OtlpSpanKind::Server as i32,
                            start_time_unix_nano: 1_000_000_000_000_000_000,
                            end_time_unix_nano: 1_000_000_000_100_000_000,
                            attributes: vec![],
                            dropped_attributes_count: 0,
                            events: vec![],
                            dropped_events_count: 0,
                            links: vec![],
                            dropped_links_count: 0,
                            status: Some(OtlpStatus {
                                message: String::new(),
                                code: OtlpStatusCode::Ok as i32,
                            }),
                        },
                        OtlpSpan {
                            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
                            trace_state: String::new(),
                            parent_span_id: vec![],
                            name: "span2".to_string(),
                            kind: OtlpSpanKind::Client as i32,
                            start_time_unix_nano: 1_000_000_000_000_000_000,
                            end_time_unix_nano: 1_000_000_000_100_000_000,
                            attributes: vec![],
                            dropped_attributes_count: 0,
                            events: vec![],
                            dropped_events_count: 0,
                            links: vec![],
                            dropped_links_count: 0,
                            status: Some(OtlpStatus {
                                message: String::new(),
                                code: OtlpStatusCode::Ok as i32,
                            }),
                        },
                    ],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/x-protobuf")
            .body(Body::from(buf))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.count_spans().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_export_traces_with_attributes() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());

        let request_data = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("attr-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![OtlpSpan {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        trace_state: String::new(),
                        parent_span_id: vec![],
                        name: "span-with-attrs".to_string(),
                        kind: OtlpSpanKind::Server as i32,
                        start_time_unix_nano: 1_000_000_000_000_000_000,
                        end_time_unix_nano: 1_000_000_000_100_000_000,
                        attributes: vec![
                            KeyValue {
                                key: "http.method".to_string(),
                                value: Some(AnyValue {
                                    value: Some(any_value::Value::StringValue("GET".to_string())),
                                }),
                            },
                            KeyValue {
                                key: "http.status_code".to_string(),
                                value: Some(AnyValue {
                                    value: Some(any_value::Value::IntValue(200)),
                                }),
                            },
                        ],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: Some(OtlpStatus {
                            message: String::new(),
                            code: OtlpStatusCode::Ok as i32,
                        }),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/x-protobuf")
            .body(Body::from(buf))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let trace = storage
            .get_trace_by_id("0102030405060708090a0b0c0d0e0f10")
            .unwrap();
        let span = &trace.spans[0];
        assert_eq!(span.attributes.get_string("http.method"), Some("GET"));
        assert_eq!(span.attributes.get_int("http.status_code"), Some(200));
    }

    #[tokio::test]
    async fn test_export_traces_http_concurrent() {
        use futures::future::join_all;
        let storage = Arc::new(Storage::new_in_memory().unwrap());

        let mut handles = vec![];
        for i in 0..10 {
            let storage_clone = storage.clone();
            let handle = tokio::spawn(async move {
                let app = create_router(storage_clone);

                let request_data = ExportTraceServiceRequest {
                    resource_spans: vec![ResourceSpans {
                        resource: Some(Resource {
                            attributes: vec![KeyValue {
                                key: "service.name".to_string(),
                                value: Some(AnyValue {
                                    value: Some(any_value::Value::StringValue(format!(
                                        "service-{}",
                                        i
                                    ))),
                                }),
                            }],
                            dropped_attributes_count: 0,
                        }),
                        scope_spans: vec![ScopeSpans {
                            scope: None,
                            spans: vec![OtlpSpan {
                                trace_id: vec![
                                    i as u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                                ],
                                span_id: vec![i as u8, 2, 3, 4, 5, 6, 7, 8],
                                trace_state: String::new(),
                                parent_span_id: vec![],
                                name: format!("span-{}", i),
                                kind: OtlpSpanKind::Server as i32,
                                start_time_unix_nano: 1_000_000_000_000_000_000,
                                end_time_unix_nano: 1_000_000_000_100_000_000,
                                attributes: vec![],
                                dropped_attributes_count: 0,
                                events: vec![],
                                dropped_events_count: 0,
                                links: vec![],
                                dropped_links_count: 0,
                                status: Some(OtlpStatus {
                                    message: String::new(),
                                    code: OtlpStatusCode::Ok as i32,
                                }),
                            }],
                            schema_url: String::new(),
                        }],
                        schema_url: String::new(),
                    }],
                };

                let mut buf = Vec::new();
                request_data.encode(&mut buf).unwrap();

                let request = Request::builder()
                    .uri("/v1/traces")
                    .method("POST")
                    .header("content-type", "application/x-protobuf")
                    .body(Body::from(buf))
                    .unwrap();

                app.oneshot(request).await.unwrap()
            });
            handles.push(handle);
        }

        let responses = join_all(handles).await;
        for response in responses {
            let resp = response.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
        assert_eq!(storage.count_spans().unwrap(), 10);
    }

    #[tokio::test]
    async fn test_export_traces_large_payload() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let mut spans = vec![];
        for i in 0..100 {
            spans.push(OtlpSpan {
                trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                span_id: vec![i as u8, i as u8, i as u8, i as u8, 5, 6, 7, 8],
                trace_state: String::new(),
                parent_span_id: vec![],
                name: format!("large-span-{}", i),
                kind: OtlpSpanKind::Internal as i32,
                start_time_unix_nano: 1_000_000_000_000_000_000 + i as u64,
                end_time_unix_nano: 1_000_000_000_100_000_000 + i as u64,
                attributes: vec![],
                dropped_attributes_count: 0,
                events: vec![],
                dropped_events_count: 0,
                links: vec![],
                dropped_links_count: 0,
                status: Some(OtlpStatus {
                    message: String::new(),
                    code: OtlpStatusCode::Ok as i32,
                }),
            });
        }

        let request_data = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("large-service".to_string())),
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

        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/x-protobuf")
            .body(Body::from(buf))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.count_spans().unwrap(), 100);
    }
}
