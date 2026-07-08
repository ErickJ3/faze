use super::attributes::Attributes;
use serde::{Deserialize, Serialize};

/// Instrumentation scope that produced a telemetry item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentationScope {
    /// Scope name (e.g., instrumentation library name)
    pub name: String,
    /// Scope version (if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Scope attributes
    #[serde(default, skip_serializing_if = "Attributes::is_empty")]
    pub attributes: Attributes,
}

impl InstrumentationScope {
    /// Build a scope from its component fields.
    #[must_use]
    pub const fn new(name: String, version: Option<String>, attributes: Attributes) -> Self {
        Self {
            name,
            version,
            attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_serde_roundtrip() {
        let mut attrs = Attributes::new();
        attrs.insert("lib.language", "rust");
        let scope = InstrumentationScope::new(
            "opentelemetry-rust".to_string(),
            Some("0.31.0".to_string()),
            attrs,
        );

        let json = serde_json::to_string(&scope).unwrap();
        let deserialized: InstrumentationScope = serde_json::from_str(&json).unwrap();
        assert_eq!(scope, deserialized);
    }

    #[test]
    fn test_scope_minimal_json() {
        let scope: InstrumentationScope = serde_json::from_str(r#"{"name":"lib"}"#).unwrap();
        assert_eq!(scope.name, "lib");
        assert_eq!(scope.version, None);
        assert!(scope.attributes.is_empty());
    }
}
