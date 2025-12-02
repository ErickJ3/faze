use crate::convert::{bytes_to_hex, convert_attributes, convert_resource};
use crate::proto::opentelemetry::proto::trace::v1::{
    ResourceSpans, Span, SpanKind as OtlpSpanKind, Status, StatusCode as OtlpSpanStatusCode,
};
use faze::models::{Span as FazeSpan, SpanKind, Status as FazeStatus, StatusCode};

/// Convert OTLP SpanKind to internal SpanKind
fn convert_span_kind(kind: i32) -> SpanKind {
    match OtlpSpanKind::try_from(kind) {
        Ok(OtlpSpanKind::Unspecified) => SpanKind::Unspecified,
        Ok(OtlpSpanKind::Internal) => SpanKind::Internal,
        Ok(OtlpSpanKind::Server) => SpanKind::Server,
        Ok(OtlpSpanKind::Client) => SpanKind::Client,
        Ok(OtlpSpanKind::Producer) => SpanKind::Producer,
        Ok(OtlpSpanKind::Consumer) => SpanKind::Consumer,
        Err(_) => SpanKind::Unspecified,
    }
}

/// Convert OTLP Span to internal Span
fn convert_span(span: &Span, service_name: Option<String>) -> FazeSpan {
    let span_id = bytes_to_hex(&span.span_id);
    let trace_id = bytes_to_hex(&span.trace_id);
    let parent_span_id = if span.parent_span_id.is_empty() {
        None
    } else {
        Some(bytes_to_hex(&span.parent_span_id))
    };

    let attributes = convert_attributes(&span.attributes);
    let kind = convert_span_kind(span.kind);
    let status = span.status.as_ref().map(convert_status).unwrap_or_default();

    FazeSpan::new(
        span_id,
        trace_id,
        parent_span_id,
        span.name.clone(),
        kind,
        span.start_time_unix_nano as i64,
        span.end_time_unix_nano as i64,
        attributes,
        status,
        service_name,
    )
}

/// Convert OTLP ResourceSpans to list of internal Spans
pub fn convert_resource_spans(resource_spans: &[ResourceSpans]) -> Vec<FazeSpan> {
    let mut spans = Vec::new();

    for rs in resource_spans {
        let service_name = rs
            .resource
            .as_ref()
            .map(convert_resource)
            .and_then(|r| r.service_name().map(|s| s.to_string()));

        for scope_spans in &rs.scope_spans {
            for span in &scope_spans.spans {
                spans.push(convert_span(span, service_name.clone()));
            }
        }
    }

    spans
}

/// Convert OTLP Status to internal Status
fn convert_status(status: &Status) -> FazeStatus {
    let code = match OtlpSpanStatusCode::try_from(status.code) {
        Ok(OtlpSpanStatusCode::Unset) => StatusCode::Unset,
        Ok(OtlpSpanStatusCode::Ok) => StatusCode::Ok,
        Ok(OtlpSpanStatusCode::Error) => StatusCode::Error,
        Err(_) => StatusCode::Unset,
    };

    let message = if status.message.is_empty() {
        None
    } else {
        Some(status.message.clone())
    };

    FazeStatus { code, message }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::proto::{
        common::v1::{AnyValue, KeyValue, any_value},
        resource::v1::Resource,
        trace::v1::ScopeSpans,
    };

    #[test]
    fn test_convert_span_kind() {
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Server as i32),
            SpanKind::Server
        );
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Client as i32),
            SpanKind::Client
        );
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Internal as i32),
            SpanKind::Internal
        );
    }

    #[test]
    fn test_convert_status() {
        let status = Status {
            code: OtlpSpanStatusCode::Ok as i32,
            message: "".to_string(),
        };
        let result = convert_status(&status);
        assert_eq!(result.code, StatusCode::Ok);
        assert_eq!(result.message, None);

        let error_status = Status {
            code: OtlpSpanStatusCode::Error as i32,
            message: "error occurred".to_string(),
        };
        let result = convert_status(&error_status);
        assert_eq!(result.code, StatusCode::Error);
        assert_eq!(result.message, Some("error occurred".to_string()));
    }

    #[test]
    fn test_convert_span_kind_all_variants() {
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Unspecified as i32),
            SpanKind::Unspecified
        );
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Internal as i32),
            SpanKind::Internal
        );
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Server as i32),
            SpanKind::Server
        );
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Client as i32),
            SpanKind::Client
        );
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Producer as i32),
            SpanKind::Producer
        );
        assert_eq!(
            convert_span_kind(OtlpSpanKind::Consumer as i32),
            SpanKind::Consumer
        );
    }

    #[test]
    fn test_convert_span_kind_invalid() {
        assert_eq!(convert_span_kind(999), SpanKind::Unspecified);
        assert_eq!(convert_span_kind(-1), SpanKind::Unspecified);
    }

    #[test]
    fn test_convert_status_all_codes() {
        let unset = Status {
            code: OtlpSpanStatusCode::Unset as i32,
            message: String::new(),
        };
        let result = convert_status(&unset);
        assert_eq!(result.code, StatusCode::Unset);

        let ok = Status {
            code: OtlpSpanStatusCode::Ok as i32,
            message: String::new(),
        };
        let result = convert_status(&ok);
        assert_eq!(result.code, StatusCode::Ok);

        let error = Status {
            code: OtlpSpanStatusCode::Error as i32,
            message: "error".to_string(),
        };
        let result = convert_status(&error);
        assert_eq!(result.code, StatusCode::Error);
        assert_eq!(result.message, Some("error".to_string()));
    }

    #[test]
    fn test_convert_status_invalid_code() {
        let status = Status {
            code: 999,
            message: String::new(),
        };
        let result = convert_status(&status);
        assert_eq!(result.code, StatusCode::Unset);
    }

    #[test]
    fn test_convert_status_empty_message() {
        let status = Status {
            code: OtlpSpanStatusCode::Error as i32,
            message: String::new(),
        };
        let result = convert_status(&status);
        assert_eq!(result.message, None);
    }

    #[test]
    fn test_convert_span_with_parent() {
        let span = Span {
            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
            parent_span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
            name: "child-span".to_string(),
            kind: OtlpSpanKind::Client as i32,
            start_time_unix_nano: 1_000_000_000,
            end_time_unix_nano: 2_000_000_000,
            attributes: vec![],
            dropped_attributes_count: 0,
            events: vec![],
            dropped_events_count: 0,
            links: vec![],
            dropped_links_count: 0,
            status: Some(Status {
                code: OtlpSpanStatusCode::Ok as i32,
                message: String::new(),
            }),
            trace_state: String::new(),
        };

        let result = convert_span(&span, Some("test-service".to_string()));
        assert_eq!(result.parent_span_id, Some("090a0b0c0d0e0f10".to_string()));
        assert!(!result.is_root());
    }

    #[test]
    fn test_convert_span_without_parent() {
        let span = Span {
            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
            parent_span_id: vec![],
            name: "root-span".to_string(),
            kind: OtlpSpanKind::Server as i32,
            start_time_unix_nano: 1_000_000_000,
            end_time_unix_nano: 2_000_000_000,
            attributes: vec![],
            dropped_attributes_count: 0,
            events: vec![],
            dropped_events_count: 0,
            links: vec![],
            dropped_links_count: 0,
            status: None,
            trace_state: String::new(),
        };

        let result = convert_span(&span, Some("test-service".to_string()));
        assert_eq!(result.parent_span_id, None);
        assert!(result.is_root());
    }

    #[test]
    fn test_convert_span_with_attributes() {
        let span = Span {
            trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
            parent_span_id: vec![],
            name: "test-span".to_string(),
            kind: OtlpSpanKind::Server as i32,
            start_time_unix_nano: 1_000_000_000,
            end_time_unix_nano: 2_000_000_000,
            attributes: vec![
                KeyValue {
                    key: "http.method".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("POST".to_string())),
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
            status: Some(Status {
                code: OtlpSpanStatusCode::Ok as i32,
                message: String::new(),
            }),
            trace_state: String::new(),
        };

        let result = convert_span(&span, Some("test-service".to_string()));
        assert_eq!(result.attributes.get_string("http.method"), Some("POST"));
        assert_eq!(result.attributes.get_int("http.status_code"), Some(200));
    }

    #[test]
    fn test_convert_resource_spans_multiple_scopes() {
        let resource_spans = vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("multi-scope".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
            }),
            scope_spans: vec![
                ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        parent_span_id: vec![],
                        name: "scope1-span".to_string(),
                        kind: OtlpSpanKind::Server as i32,
                        start_time_unix_nano: 1_000_000_000,
                        end_time_unix_nano: 2_000_000_000,
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: None,
                        trace_state: String::new(),
                    }],
                    schema_url: String::new(),
                },
                ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
                        parent_span_id: vec![],
                        name: "scope2-span".to_string(),
                        kind: OtlpSpanKind::Client as i32,
                        start_time_unix_nano: 1_500_000_000,
                        end_time_unix_nano: 2_500_000_000,
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: None,
                        trace_state: String::new(),
                    }],
                    schema_url: String::new(),
                },
            ],
            schema_url: String::new(),
        }];

        let spans = convert_resource_spans(&resource_spans);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].name, "scope1-span");
        assert_eq!(spans[1].name, "scope2-span");
        assert_eq!(spans[0].service_name, Some("multi-scope".to_string()));
        assert_eq!(spans[1].service_name, Some("multi-scope".to_string()));
    }

    #[test]
    fn test_convert_resource_spans_multiple_resources() {
        let resource_spans = vec![
            ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("service1".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        parent_span_id: vec![],
                        name: "service1-span".to_string(),
                        kind: OtlpSpanKind::Server as i32,
                        start_time_unix_nano: 1_000_000_000,
                        end_time_unix_nano: 2_000_000_000,
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: None,
                        trace_state: String::new(),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            },
            ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("service2".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
                        parent_span_id: vec![],
                        name: "service2-span".to_string(),
                        kind: OtlpSpanKind::Client as i32,
                        start_time_unix_nano: 1_500_000_000,
                        end_time_unix_nano: 2_500_000_000,
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: None,
                        trace_state: String::new(),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            },
        ];

        let spans = convert_resource_spans(&resource_spans);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].service_name, Some("service1".to_string()));
        assert_eq!(spans[1].service_name, Some("service2".to_string()));
    }

    #[test]
    fn test_convert_resource_spans_empty() {
        let resource_spans: Vec<ResourceSpans> = vec![];
        let spans = convert_resource_spans(&resource_spans);
        assert_eq!(spans.len(), 0);
    }

    #[test]
    fn test_convert_resource_spans_no_resource() {
        let resource_spans = vec![ResourceSpans {
            resource: None,
            scope_spans: vec![ScopeSpans {
                scope: None,
                spans: vec![Span {
                    trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                    span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                    parent_span_id: vec![],
                    name: "no-resource-span".to_string(),
                    kind: OtlpSpanKind::Server as i32,
                    start_time_unix_nano: 1_000_000_000,
                    end_time_unix_nano: 2_000_000_000,
                    attributes: vec![],
                    dropped_attributes_count: 0,
                    events: vec![],
                    dropped_events_count: 0,
                    links: vec![],
                    dropped_links_count: 0,
                    status: None,
                    trace_state: String::new(),
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }];

        let spans = convert_resource_spans(&resource_spans);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].service_name, None);
    }
}
