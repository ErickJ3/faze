use super::attributes::Attributes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Span kind indicates the type of span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum SpanKind {
    #[default]
    Unspecified,
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

/// Span status code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum StatusCode {
    #[default]
    Unset,
    Ok,
    Error,
}

/// Span status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub code: StatusCode,
    pub message: Option<String>,
}

impl Status {
    pub fn ok() -> Self {
        Self {
            code: StatusCode::Ok,
            message: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            code: StatusCode::Error,
            message: Some(message.into()),
        }
    }

    pub fn unset() -> Self {
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
}

impl Span {
    #[allow(clippy::too_many_arguments)]
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
        }
    }

    /// Get duration in nanoseconds
    pub fn duration_nanos(&self) -> i64 {
        self.end_time_unix_nano - self.start_time_unix_nano
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        self.duration_nanos() as f64 / 1_000_000.0
    }

    /// Get start time as DateTime
    pub fn start_time(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_nanos(self.start_time_unix_nano)
    }

    /// Get end time as DateTime
    pub fn end_time(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_nanos(self.end_time_unix_nano)
    }

    /// Check if this is a root span (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_span_id.is_none()
    }

    /// Check if span has error status
    pub fn is_error(&self) -> bool {
        self.status.code == StatusCode::Error
    }
}

#[cfg(test)]
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
