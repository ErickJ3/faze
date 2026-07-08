use super::attributes::Attributes;
use super::db_enum::impl_db_str;
use super::scope::InstrumentationScope;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Span kind indicates the type of span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpanKind {
    /// Span kind is not specified.
    #[default]
    Unspecified,
    /// Internal operation within an application.
    Internal,
    /// Inbound server handler.
    Server,
    /// Outbound client call.
    Client,
    /// Asynchronous producer of a message.
    Producer,
    /// Asynchronous consumer of a message.
    Consumer,
}

impl_db_str!(
    SpanKind {
        Unspecified,
        Internal,
        Server,
        Client,
        Producer,
        Consumer,
    },
    fallback = Unspecified
);

/// Span status code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusCode {
    /// Status not set by the producer.
    #[default]
    Unset,
    /// Operation completed successfully.
    Ok,
    /// Operation failed.
    Error,
}

/// Span status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// Status code (OK, Error, Unset).
    pub code: StatusCode,
    /// Optional descriptive message associated with the status.
    pub message: Option<String>,
}

impl Status {
    /// Build an OK status with no message.
    #[must_use]
    pub const fn ok() -> Self {
        Self {
            code: StatusCode::Ok,
            message: None,
        }
    }

    /// Build an Error status carrying a message.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            code: StatusCode::Error,
            message: Some(message.into()),
        }
    }

    /// Build an Unset status with no message.
    #[must_use]
    pub const fn unset() -> Self {
        Self {
            code: StatusCode::Unset,
            message: None,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::unset()
    }
}

/// Timed event attached to a span (e.g., an exception)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event timestamp (nanoseconds since epoch)
    pub time_unix_nano: i64,
    /// Event name
    pub name: String,
    /// Event attributes
    #[serde(default, skip_serializing_if = "Attributes::is_empty")]
    pub attributes: Attributes,
    /// Attributes dropped by the producer
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub dropped_attributes_count: u32,
}

/// Link from a span to another span, possibly in a different trace
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpanLink {
    /// Linked trace ID (hex)
    pub trace_id: String,
    /// Linked span ID (hex)
    pub span_id: String,
    /// W3C trace state of the linked span
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_state: Option<String>,
    /// Link attributes
    #[serde(default, skip_serializing_if = "Attributes::is_empty")]
    pub attributes: Attributes,
    /// Attributes dropped by the producer
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub dropped_attributes_count: u32,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

/// Represents a single span in a trace
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    /// Unique identifier for this span
    pub span_id: String,
    /// Trace ID this span belongs to
    pub trace_id: String,
    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,
    /// Name of the operation
    pub name: String,
    /// Kind of span
    pub kind: SpanKind,
    /// Start time (nanoseconds since epoch)
    pub start_time_unix_nano: i64,
    /// End time (nanoseconds since epoch)
    pub end_time_unix_nano: i64,
    /// Span attributes
    pub attributes: Attributes,
    /// Span status
    pub status: Status,
    /// Service name (denormalized from resource)
    pub service_name: Option<String>,
    /// Timed events attached to the span
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<SpanEvent>,
    /// Links to other spans
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<SpanLink>,
    /// W3C trace state
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_state: Option<String>,
    /// Attributes dropped by the producer
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub dropped_attributes_count: u32,
    /// Events dropped by the producer
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub dropped_events_count: u32,
    /// Links dropped by the producer
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub dropped_links_count: u32,
    /// Full resource attributes of the producer
    #[serde(default, skip_serializing_if = "Attributes::is_empty")]
    pub resource_attributes: Attributes,
    /// Instrumentation scope that produced the span
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<InstrumentationScope>,
}

impl Span {
    /// Build a span from its component fields.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        span_id: String,
        trace_id: String,
        parent_span_id: Option<String>,
        name: String,
        kind: SpanKind,
        start_time_unix_nano: i64,
        end_time_unix_nano: i64,
        attributes: Attributes,
        status: Status,
        service_name: Option<String>,
    ) -> Self {
        Self {
            span_id,
            trace_id,
            parent_span_id,
            name,
            kind,
            start_time_unix_nano,
            end_time_unix_nano,
            attributes,
            status,
            service_name,
            events: Vec::new(),
            links: Vec::new(),
            trace_state: None,
            dropped_attributes_count: 0,
            dropped_events_count: 0,
            dropped_links_count: 0,
            resource_attributes: Attributes::new(),
            scope: None,
        }
    }

    /// Attach timed events.
    #[must_use]
    pub fn with_events(mut self, events: Vec<SpanEvent>) -> Self {
        self.events = events;
        self
    }

    /// Attach links to other spans.
    #[must_use]
    pub fn with_links(mut self, links: Vec<SpanLink>) -> Self {
        self.links = links;
        self
    }

    /// Set the W3C trace state.
    #[must_use]
    pub fn with_trace_state(mut self, trace_state: Option<String>) -> Self {
        self.trace_state = trace_state;
        self
    }

    /// Set producer-side dropped counts (attributes, events, links).
    #[must_use]
    pub const fn with_dropped_counts(mut self, attributes: u32, events: u32, links: u32) -> Self {
        self.dropped_attributes_count = attributes;
        self.dropped_events_count = events;
        self.dropped_links_count = links;
        self
    }

    /// Attach the full resource attributes.
    #[must_use]
    pub fn with_resource_attributes(mut self, resource_attributes: Attributes) -> Self {
        self.resource_attributes = resource_attributes;
        self
    }

    /// Attach the instrumentation scope.
    #[must_use]
    pub fn with_scope(mut self, scope: Option<InstrumentationScope>) -> Self {
        self.scope = scope;
        self
    }

    /// Get duration in nanoseconds
    #[must_use]
    pub const fn duration_nanos(&self) -> i64 {
        self.end_time_unix_nano - self.start_time_unix_nano
    }

    /// Get duration in milliseconds
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn duration_ms(&self) -> f64 {
        self.duration_nanos() as f64 / 1_000_000.0
    }

    /// Get start time as `DateTime`
    #[must_use]
    pub const fn start_time(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_nanos(self.start_time_unix_nano)
    }

    /// Get end time as `DateTime`
    #[must_use]
    pub const fn end_time(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_nanos(self.end_time_unix_nano)
    }

    /// Check if this is a root span (no parent)
    #[must_use]
    pub const fn is_root(&self) -> bool {
        self.parent_span_id.is_none()
    }

    /// Check if span has error status
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.status.code == StatusCode::Error
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn create_test_span() -> Span {
        let start = 1_000_000_000_000_000_000; // 1 second in nanos
        let end = 1_000_000_000_100_000_000; // 1.1 seconds in nanos

        Span::new(
            "span123".to_string(),
            "trace456".to_string(),
            Some("parent789".to_string()),
            "test-operation".to_string(),
            SpanKind::Server,
            start,
            end,
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        )
    }

    #[test]
    fn test_span_creation() {
        let span = create_test_span();
        assert_eq!(span.span_id, "span123");
        assert_eq!(span.trace_id, "trace456");
        assert_eq!(span.parent_span_id, Some("parent789".to_string()));
        assert_eq!(span.name, "test-operation");
        assert_eq!(span.kind, SpanKind::Server);
        assert_eq!(span.service_name, Some("test-service".to_string()));
    }

    #[test]
    fn test_span_duration() {
        let span = create_test_span();
        assert_eq!(span.duration_nanos(), 100_000_000); // 100ms in nanos
        assert_eq!(span.duration_ms(), 100.0);
    }

    #[test]
    fn test_span_is_root() {
        let mut span = create_test_span();
        assert!(!span.is_root());

        span.parent_span_id = None;
        assert!(span.is_root());
    }

    #[test]
    fn test_span_is_error() {
        let mut span = create_test_span();
        assert!(!span.is_error());

        span.status = Status::error("something went wrong");
        assert!(span.is_error());
    }

    #[test]
    fn test_span_kind_default() {
        let kind = SpanKind::default();
        assert_eq!(kind, SpanKind::Unspecified);
    }

    #[test]
    fn test_status_constructors() {
        let ok = Status::ok();
        assert_eq!(ok.code, StatusCode::Ok);
        assert_eq!(ok.message, None);

        let error = Status::error("error message");
        assert_eq!(error.code, StatusCode::Error);
        assert_eq!(error.message, Some("error message".to_string()));

        let unset = Status::unset();
        assert_eq!(unset.code, StatusCode::Unset);
        assert_eq!(unset.message, None);
    }

    #[test]
    fn test_span_serde() {
        let span = create_test_span();

        let json = serde_json::to_string(&span).unwrap();
        let deserialized: Span = serde_json::from_str(&json).unwrap();
        assert_eq!(span, deserialized);
    }

    #[test]
    fn test_span_deserializes_pre_v1_json() {
        // Stored JSON written before events/links/fidelity fields existed.
        let json = r#"{
            "span_id": "s1", "trace_id": "t1", "parent_span_id": null,
            "name": "op", "kind": "SERVER",
            "start_time_unix_nano": 1, "end_time_unix_nano": 2,
            "attributes": {}, "status": {"code": "OK", "message": null},
            "service_name": "svc"
        }"#;
        let span: Span = serde_json::from_str(json).unwrap();
        assert!(span.events.is_empty());
        assert!(span.links.is_empty());
        assert_eq!(span.trace_state, None);
        assert_eq!(span.dropped_attributes_count, 0);
        assert!(span.resource_attributes.is_empty());
        assert_eq!(span.scope, None);
    }

    #[test]
    fn test_span_serde_full_fidelity() {
        let mut event_attrs = Attributes::new();
        event_attrs.insert("exception.type", "IoError");
        let span = create_test_span()
            .with_events(vec![SpanEvent {
                time_unix_nano: 1_000_000_000_000_000_500,
                name: "exception".to_string(),
                attributes: event_attrs,
                dropped_attributes_count: 1,
            }])
            .with_links(vec![SpanLink {
                trace_id: "other-trace".to_string(),
                span_id: "other-span".to_string(),
                trace_state: Some("vendor=1".to_string()),
                attributes: Attributes::new(),
                dropped_attributes_count: 0,
            }])
            .with_trace_state(Some("vendor=2".to_string()))
            .with_dropped_counts(1, 2, 3)
            .with_scope(Some(InstrumentationScope::new(
                "test-lib".to_string(),
                Some("1.0".to_string()),
                Attributes::new(),
            )));

        let json = serde_json::to_string(&span).unwrap();
        let deserialized: Span = serde_json::from_str(&json).unwrap();
        assert_eq!(span, deserialized);
    }

    #[test]
    fn test_span_with_attributes() {
        let mut attrs = Attributes::new();
        attrs.insert("http.method", "GET");
        attrs.insert("http.status_code", 200i64);

        let span = Span::new(
            "span1".to_string(),
            "trace1".to_string(),
            None,
            "GET /api/users".to_string(),
            SpanKind::Server,
            1_000_000_000,
            2_000_000_000,
            attrs.clone(),
            Status::ok(),
            Some("api".to_string()),
        );

        assert_eq!(span.attributes.get_string("http.method"), Some("GET"));
        assert_eq!(span.attributes.get_int("http.status_code"), Some(200));
    }
}
