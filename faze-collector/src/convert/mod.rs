//! Conversion helpers from OTLP protobuf types into Faze domain types.
//!
//! `schema_url` (resource- and scope-level) is consciously not persisted: it
//! carries no value for a local dev tool and can be added additively later.

use crate::proto::opentelemetry::proto::{
    common::v1::{AnyValue, InstrumentationScope, KeyValue, any_value},
    resource::v1::Resource,
};
use faze::models::{
    AttributeValue, Attributes, InstrumentationScope as FazeScope, Resource as FazeResource,
};

/// Log-record conversion helpers.
pub mod logs;
/// Metric conversion helpers.
pub mod metrics;
/// Trace/span conversion helpers.
pub mod traces;

/// Convert OTLP `AnyValue` to internal `AttributeValue`
#[must_use]
pub fn convert_any_value(value: &AnyValue) -> Option<AttributeValue> {
    value.value.as_ref().map(|v| match v {
        any_value::Value::StringValue(s) => AttributeValue::String(s.clone()),
        any_value::Value::BoolValue(b) => AttributeValue::Bool(*b),
        any_value::Value::IntValue(i) => AttributeValue::Int(*i),
        any_value::Value::DoubleValue(d) => AttributeValue::Double(*d),
        any_value::Value::BytesValue(b) => AttributeValue::Bytes(b.clone()),
        any_value::Value::ArrayValue(arr) => {
            let values: Vec<AttributeValue> =
                arr.values.iter().filter_map(convert_any_value).collect();
            AttributeValue::Array(values)
        }
        any_value::Value::KvlistValue(kvlist) => {
            let map = kvlist
                .values
                .iter()
                .filter_map(|kv| {
                    kv.value
                        .as_ref()
                        .and_then(convert_any_value)
                        .map(|v| (kv.key.clone(), v))
                })
                .collect();
            AttributeValue::Map(map)
        }
    })
}

/// Convert OTLP `AnyValue` to a string
#[must_use]
pub fn convert_any_value_to_string(value: &AnyValue) -> Option<String> {
    value.value.as_ref().map(|v| match v {
        any_value::Value::StringValue(s) => s.clone(),
        any_value::Value::BoolValue(b) => b.to_string(),
        any_value::Value::IntValue(i) => i.to_string(),
        any_value::Value::DoubleValue(d) => d.to_string(),
        any_value::Value::BytesValue(b) => bytes_to_hex(b),
        any_value::Value::ArrayValue(arr) => {
            let values: Vec<String> = arr
                .values
                .iter()
                .filter_map(convert_any_value_to_string)
                .collect();
            format!("[{}]", values.join(","))
        }
        any_value::Value::KvlistValue(kvlist) => {
            let entries: Vec<String> = kvlist
                .values
                .iter()
                .map(|kv| {
                    let value = kv
                        .value
                        .as_ref()
                        .and_then(convert_any_value_to_string)
                        .unwrap_or_default();
                    format!("{}={value}", kv.key)
                })
                .collect();
            format!("{{{}}}", entries.join(","))
        }
    })
}

/// Convert OTLP `KeyValue` list to `Attributes`
#[must_use]
pub fn convert_attributes(kvs: &[KeyValue]) -> Attributes {
    kvs.iter()
        .filter_map(|kv| {
            kv.value
                .as_ref()
                .and_then(convert_any_value)
                .map(|v| (kv.key.clone(), v))
        })
        .collect()
}

/// Convert OTLP `Resource` to internal `Resource`
#[must_use]
pub fn convert_resource(resource: &Resource) -> FazeResource {
    let attributes = convert_attributes(&resource.attributes);
    FazeResource::new(attributes)
}

/// Resource context shared by every signal in an OTLP resource group.
pub(crate) struct ResourceContext {
    /// Denormalized `service.name`.
    pub service_name: Option<String>,
    /// Full converted resource attributes.
    pub attributes: Attributes,
}

/// Convert an optional OTLP resource into the shared per-resource context.
pub(crate) fn convert_resource_context(resource: Option<&Resource>) -> ResourceContext {
    let attributes = resource
        .map(|r| convert_attributes(&r.attributes))
        .unwrap_or_default();
    let service_name = attributes.get_string("service.name").map(str::to_string);
    ResourceContext {
        service_name,
        attributes,
    }
}

/// Convert an optional OTLP instrumentation scope; empty scopes become `None`.
pub(crate) fn convert_scope(scope: Option<&InstrumentationScope>) -> Option<FazeScope> {
    let scope = scope?;
    if scope.name.is_empty() && scope.version.is_empty() && scope.attributes.is_empty() {
        return None;
    }
    let version = if scope.version.is_empty() {
        None
    } else {
        Some(scope.version.clone())
    };
    Some(FazeScope::new(
        scope.name.clone(),
        version,
        convert_attributes(&scope.attributes),
    ))
}

/// Hex-encode an ID, mapping empty bytes to `None`.
pub(crate) fn id_to_hex(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        None
    } else {
        Some(bytes_to_hex(bytes))
    }
}

/// Convert bytes to hex string.
#[must_use]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Parse a strict lowercase/uppercase hex string of even length.
#[must_use]
pub fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    let digit = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    };
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| Some(digit(pair[0])? << 4 | digit(pair[1])?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

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
        let AttributeValue::Array(arr) = result else {
            unreachable!("expected array value, got non-array variant");
        };
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], AttributeValue::String("item1".to_string()));
        assert_eq!(arr[1], AttributeValue::Int(42));
    }

    #[test]
    fn test_convert_any_value_empty_array() {
        let value = AnyValue {
            value: Some(any_value::Value::ArrayValue(
                crate::proto::opentelemetry::proto::common::v1::ArrayValue { values: vec![] },
            )),
        };
        let result = convert_any_value(&value).unwrap();
        let AttributeValue::Array(arr) = result else {
            unreachable!("expected array value, got non-array variant");
        };
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_hex_to_bytes() {
        assert_eq!(hex_to_bytes("1234abcd"), Some(vec![0x12, 0x34, 0xab, 0xcd]));
        assert_eq!(hex_to_bytes("1234ABCD"), Some(vec![0x12, 0x34, 0xab, 0xcd]));
        assert_eq!(hex_to_bytes(""), Some(vec![]));
        assert_eq!(hex_to_bytes("123"), None); // odd length
        assert_eq!(hex_to_bytes("12zz"), None); // non-hex
    }

    #[test]
    fn test_convert_any_value_kvlist() {
        use crate::proto::opentelemetry::proto::common::v1::KeyValueList;
        let value = AnyValue {
            value: Some(any_value::Value::KvlistValue(KeyValueList {
                values: vec![KeyValue {
                    key: "nested".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::IntValue(7)),
                    }),
                }],
            })),
        };
        let result = convert_any_value(&value).unwrap();
        let AttributeValue::Map(map) = result else {
            unreachable!("expected map value, got non-map variant");
        };
        assert_eq!(map.get("nested"), Some(&AttributeValue::Int(7)));
    }

    #[test]
    fn test_convert_any_value_to_string_kvlist() {
        use crate::proto::opentelemetry::proto::common::v1::KeyValueList;
        let value = AnyValue {
            value: Some(any_value::Value::KvlistValue(KeyValueList {
                values: vec![KeyValue {
                    key: "k".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("v".to_string())),
                    }),
                }],
            })),
        };
        assert_eq!(
            convert_any_value_to_string(&value),
            Some("{k=v}".to_string())
        );
    }

    #[test]
    fn test_convert_scope() {
        let scope = InstrumentationScope {
            name: "my-lib".to_string(),
            version: "1.0".to_string(),
            attributes: vec![],
            dropped_attributes_count: 0,
        };
        let converted = convert_scope(Some(&scope)).unwrap();
        assert_eq!(converted.name, "my-lib");
        assert_eq!(converted.version, Some("1.0".to_string()));

        // Empty scope collapses to None.
        let empty = InstrumentationScope::default();
        assert_eq!(convert_scope(Some(&empty)), None);
        assert_eq!(convert_scope(None), None);
    }

    #[test]
    fn test_convert_resource_context() {
        let resource = Resource {
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
                        value: Some(any_value::Value::StringValue("1.2".to_string())),
                    }),
                },
            ],
            dropped_attributes_count: 0,
        };
        let ctx = convert_resource_context(Some(&resource));
        assert_eq!(ctx.service_name, Some("svc".to_string()));
        assert_eq!(ctx.attributes.get_string("service.version"), Some("1.2"));

        let empty = convert_resource_context(None);
        assert_eq!(empty.service_name, None);
        assert!(empty.attributes.is_empty());
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
}
