use crate::proto::opentelemetry::proto::{
    common::v1::{AnyValue, KeyValue, any_value},
    resource::v1::Resource,
    trace::v1::{
        ResourceSpans, Span, SpanKind as OtlpSpanKind, Status, StatusCode as OtlpStatusCode,
    },
};
use glint::models::{
    AttributeValue, Attributes, Resource as GlintResource, Span as GlintSpan, SpanKind,
    Status as GlintStatus, StatusCode,
};

/// Convert OTLP AnyValue to internal AttributeValue
fn convert_any_value(value: &AnyValue) -> Option<AttributeValue> {
    value.value.as_ref().and_then(|v| match v {
        any_value::Value::StringValue(s) => Some(AttributeValue::String(s.clone())),
        any_value::Value::BoolValue(b) => Some(AttributeValue::Bool(*b)),
        any_value::Value::IntValue(i) => Some(AttributeValue::Int(*i)),
        any_value::Value::DoubleValue(d) => Some(AttributeValue::Double(*d)),
        any_value::Value::BytesValue(b) => Some(AttributeValue::Bytes(b.clone())),
        any_value::Value::ArrayValue(arr) => {
            let values: Vec<AttributeValue> =
                arr.values.iter().filter_map(convert_any_value).collect();
            Some(AttributeValue::Array(values))
        }
        any_value::Value::KvlistValue(_) => None, // Not supported for now
    })
}

/// Convert OTLP KeyValue list to Attributes
fn convert_attributes(kvs: &[KeyValue]) -> Attributes {
    kvs.iter()
        .filter_map(|kv| {
            kv.value
                .as_ref()
                .and_then(convert_any_value)
                .map(|v| (kv.key.clone(), v))
        })
        .collect()
}

/// Convert OTLP Resource to internal Resource
fn convert_resource(resource: &Resource) -> GlintResource {
    let attributes = convert_attributes(&resource.attributes);
    GlintResource::new(attributes)
}

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

/// Convert OTLP Status to internal Status
fn convert_status(status: &Status) -> GlintStatus {
    let code = match OtlpStatusCode::try_from(status.code) {
        Ok(OtlpStatusCode::Unset) => StatusCode::Unset,
        Ok(OtlpStatusCode::Ok) => StatusCode::Ok,
        Ok(OtlpStatusCode::Error) => StatusCode::Error,
        Err(_) => StatusCode::Unset,
    };

    let message = if status.message.is_empty() {
        None
    } else {
        Some(status.message.clone())
    };

    GlintStatus { code, message }
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Convert OTLP Span to internal Span
fn convert_span(span: &Span, service_name: Option<String>) -> GlintSpan {
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

    GlintSpan::new(
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
pub fn convert_resource_spans(resource_spans: &[ResourceSpans]) -> Vec<GlintSpan> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::proto::trace::v1::ScopeSpans;
    use std;

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
    fn test_bytes_to_hex() {
        assert_eq!(bytes_to_hex(&[0x12, 0x34, 0xab, 0xcd]), "1234abcd");
        assert_eq!(bytes_to_hex(&[]), "");
    }

    #[test]
    fn test_convert_any_value_string() {
        let value = AnyValue {
            value: Some(any_value::Value::StringValue("test".to_string())),
        };
        let result = convert_any_value(&value).unwrap();
        assert_eq!(result, AttributeValue::String("test".to_string()));
    }

    #[test]
    fn test_convert_any_value_int() {
        let value = AnyValue {
            value: Some(any_value::Value::IntValue(42)),
        };
        let result = convert_any_value(&value).unwrap();
        assert_eq!(result, AttributeValue::Int(42));
    }

    #[test]
    fn test_convert_any_value_bool() {
        let value = AnyValue {
            value: Some(any_value::Value::BoolValue(true)),
        };
        let result = convert_any_value(&value).unwrap();
        assert_eq!(result, AttributeValue::Bool(true));
    }

    #[test]
    fn test_convert_attributes() {
        let kvs = vec![
            KeyValue {
                key: "name".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("test".to_string())),
                }),
            },
            KeyValue {
                key: "count".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::IntValue(10)),
                }),
            },
        ];

        let attrs = convert_attributes(&kvs);
        assert_eq!(attrs.get_string("name"), Some("test"));
        assert_eq!(attrs.get_int("count"), Some(10));
    }

    #[test]
    fn test_convert_status() {
        let status = Status {
            code: OtlpStatusCode::Ok as i32,
            message: "".to_string(),
        };
        let result = convert_status(&status);
        assert_eq!(result.code, StatusCode::Ok);
        assert_eq!(result.message, None);

        let error_status = Status {
            code: OtlpStatusCode::Error as i32,
            message: "error occurred".to_string(),
        };
        let result = convert_status(&error_status);
        assert_eq!(result.code, StatusCode::Error);
        assert_eq!(result.message, Some("error occurred".to_string()));
    }

    #[test]
    fn test_convert_resource() {
        let resource = Resource {
            attributes: vec![KeyValue {
                key: "service.name".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("my-service".to_string())),
                }),
            }],
            dropped_attributes_count: 0,
        };

        let result = convert_resource(&resource);
        assert_eq!(result.service_name(), Some("my-service"));
    }

    #[test]
    fn test_convert_any_value_double() {
        let value = AnyValue {
            value: Some(any_value::Value::DoubleValue(std::f64::consts::PI)),
        };
        let result = convert_any_value(&value).unwrap();
        assert_eq!(result, AttributeValue::Double(std::f64::consts::PI));
    }

    #[test]
    fn test_convert_any_value_bytes() {
        let bytes = vec![0x01, 0x02, 0x03];
        let value = AnyValue {
            value: Some(any_value::Value::BytesValue(bytes.clone())),
        };
        let result = convert_any_value(&value).unwrap();
        assert_eq!(result, AttributeValue::Bytes(bytes));
    }

    #[test]
    fn test_convert_any_value_array() {
        let value = AnyValue {
            value: Some(any_value::Value::ArrayValue(
                crate::proto::opentelemetry::proto::common::v1::ArrayValue {
                    values: vec![
                        AnyValue {
                            value: Some(any_value::Value::StringValue("item1".to_string())),
                        },
                        AnyValue {
                            value: Some(any_value::Value::IntValue(42)),
                        },
                    ],
                },
            )),
        };
        let result = convert_any_value(&value).unwrap();
        if let AttributeValue::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], AttributeValue::String("item1".to_string()));
            assert_eq!(arr[1], AttributeValue::Int(42));
        } else {
            panic!("Expected array value");
        }
    }

    #[test]
    fn test_convert_any_value_empty_array() {
        let value = AnyValue {
            value: Some(any_value::Value::ArrayValue(
                crate::proto::opentelemetry::proto::common::v1::ArrayValue { values: vec![] },
            )),
        };
        let result = convert_any_value(&value).unwrap();
        if let AttributeValue::Array(arr) = result {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected array value");
        }
    }

    #[test]
    fn test_convert_any_value_none() {
        let value = AnyValue { value: None };
        let result = convert_any_value(&value);
        assert!(result.is_none());
    }

    #[test]
    fn test_convert_attributes_empty() {
        let kvs: Vec<KeyValue> = vec![];
        let attrs = convert_attributes(&kvs);
        assert!(attrs.is_empty());
    }

    #[test]
    fn test_convert_attributes_mixed_types() {
        let kvs = vec![
            KeyValue {
                key: "string_key".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("value".to_string())),
                }),
            },
            KeyValue {
                key: "int_key".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::IntValue(123)),
                }),
            },
            KeyValue {
                key: "bool_key".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::BoolValue(false)),
                }),
            },
            KeyValue {
                key: "double_key".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::DoubleValue(std::f64::consts::E)),
                }),
            },
        ];

        let attrs = convert_attributes(&kvs);
        assert_eq!(attrs.get_string("string_key"), Some("value"));
        assert_eq!(attrs.get_int("int_key"), Some(123));
        assert_eq!(attrs.get_bool("bool_key"), Some(false));
        assert_eq!(attrs.get_double("double_key"), Some(std::f64::consts::E));
    }

    #[test]
    fn test_convert_attributes_with_none_values() {
        let kvs = vec![
            KeyValue {
                key: "valid".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("test".to_string())),
                }),
            },
            KeyValue {
                key: "none_value".to_string(),
                value: None,
            },
            KeyValue {
                key: "empty_value".to_string(),
                value: Some(AnyValue { value: None }),
            },
        ];

        let attrs = convert_attributes(&kvs);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs.get_string("valid"), Some("test"));
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
            code: OtlpStatusCode::Unset as i32,
            message: String::new(),
        };
        let result = convert_status(&unset);
        assert_eq!(result.code, StatusCode::Unset);

        let ok = Status {
            code: OtlpStatusCode::Ok as i32,
            message: String::new(),
        };
        let result = convert_status(&ok);
        assert_eq!(result.code, StatusCode::Ok);

        let error = Status {
            code: OtlpStatusCode::Error as i32,
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
            code: OtlpStatusCode::Error as i32,
            message: String::new(),
        };
        let result = convert_status(&status);
        assert_eq!(result.message, None);
    }

    #[test]
    fn test_bytes_to_hex_empty() {
        assert_eq!(bytes_to_hex(&[]), "");
    }

    #[test]
    fn test_bytes_to_hex_single_byte() {
        assert_eq!(bytes_to_hex(&[0xFF]), "ff");
    }

    #[test]
    fn test_bytes_to_hex_standard_trace_id() {
        let trace_id = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        assert_eq!(bytes_to_hex(&trace_id), "000102030405060708090a0b0c0d0e0f");
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
                code: OtlpStatusCode::Ok as i32,
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
                code: OtlpStatusCode::Ok as i32,
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
