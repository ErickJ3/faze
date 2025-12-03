use crate::proto::opentelemetry::proto::{
    common::v1::{AnyValue, KeyValue, any_value},
    resource::v1::Resource,
};
use faze::models::{AttributeValue, Attributes, Resource as FazeResource};

pub mod logs;
pub mod metrics;
pub mod traces;

/// Convert OTLP AnyValue to internal AttributeValue
pub fn convert_any_value(value: &AnyValue) -> Option<AttributeValue> {
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

/// Convert OTLP AnyValue to a string
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
        any_value::Value::KvlistValue(_) => "<kvlist unsupported>".to_string(),
    })
}

/// Convert OTLP KeyValue list to Attributes
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

/// Convert OTLP Resource to internal Resource
pub fn convert_resource(resource: &Resource) -> FazeResource {
    let attributes = convert_attributes(&resource.attributes);
    FazeResource::new(attributes)
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
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
}
