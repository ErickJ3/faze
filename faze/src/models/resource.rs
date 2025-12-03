use super::attributes::Attributes;
use serde::{Deserialize, Serialize};

/// Resource information that identifies the entity producing telemetry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Resource attributes (e.g., service.name, service.version, host.name)
    pub attributes: Attributes,
}

impl Resource {
    pub fn new(attributes: Attributes) -> Self {
        Self { attributes }
    }

    pub fn empty() -> Self {
        Self {
            attributes: Attributes::new(),
        }
    }

    /// Get the service name from resource attributes
    pub fn service_name(&self) -> Option<&str> {
        self.attributes.get_string("service.name")
    }

    /// Get the service version from resource attributes
    pub fn service_version(&self) -> Option<&str> {
        self.attributes.get_string("service.version")
    }

    /// Get the service instance ID from resource attributes
    pub fn service_instance_id(&self) -> Option<&str> {
        self.attributes.get_string("service.instance.id")
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_new() {
        let mut attrs = Attributes::new();
        attrs.insert("service.name", "test-service");
        attrs.insert("service.version", "1.0.0");

        let resource = Resource::new(attrs.clone());
        assert_eq!(resource.attributes, attrs);
    }

    #[test]
    fn test_resource_empty() {
        let resource = Resource::empty();
        assert!(resource.attributes.is_empty());
    }

    #[test]
    fn test_resource_service_name() {
        let mut attrs = Attributes::new();
        attrs.insert("service.name", "my-api");

        let resource = Resource::new(attrs);
        assert_eq!(resource.service_name(), Some("my-api"));
    }

    #[test]
    fn test_resource_service_version() {
        let mut attrs = Attributes::new();
        attrs.insert("service.version", "2.0.0");

        let resource = Resource::new(attrs);
        assert_eq!(resource.service_version(), Some("2.0.0"));
    }

    #[test]
    fn test_resource_service_instance_id() {
        let mut attrs = Attributes::new();
        attrs.insert("service.instance.id", "instance-123");

        let resource = Resource::new(attrs);
        assert_eq!(resource.service_instance_id(), Some("instance-123"));
    }

    #[test]
    fn test_resource_serde() {
        let mut attrs = Attributes::new();
        attrs.insert("service.name", "test-service");
        attrs.insert("service.version", "1.0.0");

        let resource = Resource::new(attrs);

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(resource, deserialized);
    }
}
