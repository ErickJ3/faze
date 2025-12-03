use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents OTLP attribute values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Array(Vec<AttributeValue>),
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self {
        AttributeValue::Int(i)
    }
}

impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self {
        AttributeValue::Double(f)
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}

/// Collection of attributes (key-value pairs)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Attributes(HashMap<String, AttributeValue>);

impl Attributes {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<AttributeValue>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&AttributeValue> {
        self.0.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &AttributeValue)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.0.get(key) {
            Some(AttributeValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.0.get(key) {
            Some(AttributeValue::Int(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.0.get(key) {
            Some(AttributeValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_double(&self, key: &str) -> Option<f64> {
        match self.0.get(key) {
            Some(AttributeValue::Double(d)) => Some(*d),
            _ => None,
        }
    }
}

impl From<HashMap<String, AttributeValue>> for Attributes {
    fn from(map: HashMap<String, AttributeValue>) -> Self {
        Self(map)
    }
}

impl FromIterator<(String, AttributeValue)> for Attributes {
    fn from_iter<T: IntoIterator<Item = (String, AttributeValue)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_value_from_string() {
        let val: AttributeValue = "test".into();
        assert_eq!(val, AttributeValue::String("test".to_string()));
    }

    #[test]
    fn test_attribute_value_from_int() {
        let val: AttributeValue = 42i64.into();
        assert_eq!(val, AttributeValue::Int(42));
    }

    #[test]
    fn test_attribute_value_from_bool() {
        let val: AttributeValue = true.into();
        assert_eq!(val, AttributeValue::Bool(true));
    }

    #[test]
    fn test_attributes_insert_and_get() {
        let mut attrs = Attributes::new();
        attrs.insert("key", "value");
        assert_eq!(attrs.get_string("key"), Some("value"));
    }

    #[test]
    fn test_attributes_get_typed() {
        let mut attrs = Attributes::new();
        attrs.insert("string", "value");
        attrs.insert("int", 42i64);
        attrs.insert("bool", true);

        assert_eq!(attrs.get_string("string"), Some("value"));
        assert_eq!(attrs.get_int("int"), Some(42));
        assert_eq!(attrs.get_bool("bool"), Some(true));
    }

    #[test]
    fn test_attributes_len() {
        let mut attrs = Attributes::new();
        assert_eq!(attrs.len(), 0);
        assert!(attrs.is_empty());

        attrs.insert("key", "value");
        assert_eq!(attrs.len(), 1);
        assert!(!attrs.is_empty());
    }

    #[test]
    fn test_attributes_serde() {
        let mut attrs = Attributes::new();
        attrs.insert("service", "api");
        attrs.insert("port", 8080i64);
        attrs.insert("enabled", true);

        let json = serde_json::to_string(&attrs).unwrap();
        let deserialized: Attributes = serde_json::from_str(&json).unwrap();
        assert_eq!(attrs, deserialized);
    }
}
