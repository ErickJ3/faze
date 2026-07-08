use crate::convert::{
    ResourceContext, bytes_to_hex, convert_attributes, convert_resource_context, convert_scope,
    id_to_hex,
};
use crate::proto::opentelemetry::proto::trace::v1::{
    Event, Link, ResourceSpans, Span, SpanKind as OtlpSpanKind, Status,
    StatusCode as OtlpSpanStatusCode,
};
use faze::models::{
    InstrumentationScope as FazeScope, Span as FazeSpan, SpanEvent, SpanKind, SpanLink,
    Status as FazeStatus, StatusCode,
};

/// Convert OTLP `SpanKind` to internal `SpanKind`
fn convert_span_kind(kind: i32) -> SpanKind {
    match OtlpSpanKind::try_from(kind) {
        Ok(OtlpSpanKind::Internal) => SpanKind::Internal,
        Ok(OtlpSpanKind::Server) => SpanKind::Server,
        Ok(OtlpSpanKind::Client) => SpanKind::Client,
        Ok(OtlpSpanKind::Producer) => SpanKind::Producer,
        Ok(OtlpSpanKind::Consumer) => SpanKind::Consumer,
        Ok(OtlpSpanKind::Unspecified) | Err(_) => SpanKind::Unspecified,
    }
}

/// Convert OTLP span `Event` to internal `SpanEvent`
#[allow(clippy::cast_possible_wrap)]
fn convert_span_event(event: &Event) -> SpanEvent {
    SpanEvent {
        time_unix_nano: event.time_unix_nano as i64,
        name: event.name.clone(),
        attributes: convert_attributes(&event.attributes),
        dropped_attributes_count: event.dropped_attributes_count,
    }
}

/// Convert OTLP span `Link` to internal `SpanLink`
fn convert_span_link(link: &Link) -> SpanLink {
    let trace_state = if link.trace_state.is_empty() {
        None
    } else {
        Some(link.trace_state.clone())
    };
    SpanLink {
        trace_id: bytes_to_hex(&link.trace_id),
        span_id: bytes_to_hex(&link.span_id),
        trace_state,
        attributes: convert_attributes(&link.attributes),
        dropped_attributes_count: link.dropped_attributes_count,
    }
}

/// Convert OTLP `Span` to internal `Span`
#[allow(clippy::cast_possible_wrap)]
fn convert_span(span: &Span, resource: &ResourceContext, scope: Option<&FazeScope>) -> FazeSpan {
    let span_id = bytes_to_hex(&span.span_id);
    let trace_id = bytes_to_hex(&span.trace_id);
    let parent_span_id = id_to_hex(&span.parent_span_id);

    let attributes = convert_attributes(&span.attributes);
    let kind = convert_span_kind(span.kind);
    let status = span.status.as_ref().map(convert_status).unwrap_or_default();

    let trace_state = if span.trace_state.is_empty() {
        None
    } else {
        Some(span.trace_state.clone())
    };

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
        resource.service_name.clone(),
    )
    .with_events(span.events.iter().map(convert_span_event).collect())
    .with_links(span.links.iter().map(convert_span_link).collect())
    .with_trace_state(trace_state)
    .with_dropped_counts(
        span.dropped_attributes_count,
        span.dropped_events_count,
        span.dropped_links_count,
    )
    .with_resource_attributes(resource.attributes.clone())
    .with_scope(scope.cloned())
}

/// Convert OTLP `ResourceSpans` to list of internal `Span`s
#[must_use]
pub fn convert_resource_spans(resource_spans: &[ResourceSpans]) -> Vec<FazeSpan> {
    let mut spans = Vec::new();

    for rs in resource_spans {
        let resource = convert_resource_context(rs.resource.as_ref());

        for scope_spans in &rs.scope_spans {
            let scope = convert_scope(scope_spans.scope.as_ref());
            for span in &scope_spans.spans {
                spans.push(convert_span(span, &resource, scope.as_ref()));
            }
        }
    }

    spans
}

/// Convert OTLP `Status` to internal `Status`
fn convert_status(status: &Status) -> FazeStatus {
    let code = match OtlpSpanStatusCode::try_from(status.code) {
        Ok(OtlpSpanStatusCode::Ok) => StatusCode::Ok,
        Ok(OtlpSpanStatusCode::Error) => StatusCode::Error,
        Ok(OtlpSpanStatusCode::Unset) | Err(_) => StatusCode::Unset,
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
    use faze::models::Attributes;

    fn test_resource_ctx() -> ResourceContext {
        ResourceContext {
            service_name: Some("test-service".to_string()),
            attributes: Attributes::new(),
        }
    }
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
            message: String::new(),
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
    fn test_convert_span_events_links_and_state() {
        let span = Span {
            trace_id: vec![1; 16],
            span_id: vec![2; 8],
            parent_span_id: vec![],
            name: "with-fidelity".to_string(),
            kind: OtlpSpanKind::Internal as i32,
            start_time_unix_nano: 1_000,
            end_time_unix_nano: 2_000,
            attributes: vec![],
            dropped_attributes_count: 1,
            events: vec![Event {
                time_unix_nano: 1_500,
                name: "exception".to_string(),
                attributes: vec![KeyValue {
                    key: "exception.message".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("boom".to_string())),
                    }),
                }],
                dropped_attributes_count: 4,
            }],
            dropped_events_count: 2,
            links: vec![Link {
                trace_id: vec![0xaa; 16],
                span_id: vec![0xbb; 8],
                trace_state: "vendor=1".to_string(),
                flags: 0,
                attributes: vec![],
                dropped_attributes_count: 0,
            }],
            dropped_links_count: 3,
            status: None,
            trace_state: "vendor=2".to_string(),
            flags: 0,
        };

        let result = convert_span(&span, &test_resource_ctx(), None);

        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].name, "exception");
        assert_eq!(result.events[0].time_unix_nano, 1_500);
        assert_eq!(
            result.events[0].attributes.get_string("exception.message"),
            Some("boom")
        );
        assert_eq!(result.events[0].dropped_attributes_count, 4);

        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0].trace_id, "aa".repeat(16));
        assert_eq!(result.links[0].span_id, "bb".repeat(8));
        assert_eq!(result.links[0].trace_state, Some("vendor=1".to_string()));

        assert_eq!(result.trace_state, Some("vendor=2".to_string()));
        assert_eq!(result.dropped_attributes_count, 1);
        assert_eq!(result.dropped_events_count, 2);
        assert_eq!(result.dropped_links_count, 3);
    }

    #[test]
    fn test_convert_resource_spans_carries_resource_and_scope() {
        use crate::proto::opentelemetry::proto::common::v1::InstrumentationScope;

        let resource_spans = vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![
                    KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("svc".to_string())),
                        }),
                    },
                    KeyValue {
                        key: "service.version".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("3.1".to_string())),
                        }),
                    },
                ],
                dropped_attributes_count: 0,
            }),
            scope_spans: vec![ScopeSpans {
                scope: Some(InstrumentationScope {
                    name: "my-lib".to_string(),
                    version: "0.9".to_string(),
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }),
                spans: vec![Span {
                    trace_id: vec![1; 16],
                    span_id: vec![2; 8],
                    name: "op".to_string(),
                    ..Default::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }];

        let spans = convert_resource_spans(&resource_spans);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].service_name, Some("svc".to_string()));
        assert_eq!(
            spans[0].resource_attributes.get_string("service.version"),
            Some("3.1")
        );
        let scope = spans[0].scope.as_ref().unwrap();
        assert_eq!(scope.name, "my-lib");
        assert_eq!(scope.version, Some("0.9".to_string()));
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
            flags: 0,
        };

        let result = convert_span(&span, &test_resource_ctx(), None);
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
            flags: 0,
        };

        let result = convert_span(&span, &test_resource_ctx(), None);
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
            flags: 0,
        };

        let result = convert_span(&span, &test_resource_ctx(), None);
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
                        flags: 0,
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
                        flags: 0,
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
                        flags: 0,
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
                        flags: 0,
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
                    flags: 0,
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
