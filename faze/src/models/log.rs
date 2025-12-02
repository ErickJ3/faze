use super::attributes::Attributes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Log severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum SeverityLevel {
    #[default]
    Unspecified = 0,
    Trace = 1,
    Trace2 = 2,
    Trace3 = 3,
    Trace4 = 4,
    Debug = 5,
    Debug2 = 6,
    Debug3 = 7,
    Debug4 = 8,
    Info = 9,
    Info2 = 10,
    Info3 = 11,
    Info4 = 12,
    Warn = 13,
    Warn2 = 14,
    Warn3 = 15,
    Warn4 = 16,
    Error = 17,
    Error2 = 18,
    Error3 = 19,
    Error4 = 20,
    Fatal = 21,
    Fatal2 = 22,
    Fatal3 = 23,
    Fatal4 = 24,
}

impl SeverityLevel {
    /// Get a simplified string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unspecified => "UNSPECIFIED",
            Self::Trace | Self::Trace2 | Self::Trace3 | Self::Trace4 => "TRACE",
            Self::Debug | Self::Debug2 | Self::Debug3 | Self::Debug4 => "DEBUG",
            Self::Info | Self::Info2 | Self::Info3 | Self::Info4 => "INFO",
            Self::Warn | Self::Warn2 | Self::Warn3 | Self::Warn4 => "WARN",
            Self::Error | Self::Error2 | Self::Error3 | Self::Error4 => "ERROR",
            Self::Fatal | Self::Fatal2 | Self::Fatal3 | Self::Fatal4 => "FATAL",
        }
    }
}

/// Represents a log entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Log {
    /// Timestamp (nanoseconds since epoch)
    pub time_unix_nano: i64,
    /// Severity level
    pub severity_level: SeverityLevel,
    /// Optional severity text (e.g., "INFO", "ERROR")
    pub severity_text: Option<String>,
    /// Log message body
    pub body: String,
    /// Log attributes
    pub attributes: Attributes,
    /// Trace ID this log is associated with (if any)
    pub trace_id: Option<String>,
    /// Span ID this log is associated with (if any)
    pub span_id: Option<String>,
    /// Service name (denormalized from resource)
    pub service_name: Option<String>,
}

impl Log {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        time_unix_nano: i64,
        severity_level: SeverityLevel,
        severity_text: Option<String>,
        body: String,
        attributes: Attributes,
        trace_id: Option<String>,
        span_id: Option<String>,
        service_name: Option<String>,
    ) -> Self {
        Self {
            time_unix_nano,
            severity_level,
            severity_text,
            body,
            attributes,
            trace_id,
            span_id,
            service_name,
        }
    }

    /// Get timestamp as DateTime
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_nanos(self.time_unix_nano)
    }

    /// Check if log is correlated with a trace
    pub fn is_correlated(&self) -> bool {
        self.trace_id.is_some() && self.span_id.is_some()
    }

    /// Check if log level is error or fatal
    pub fn is_error(&self) -> bool {
        matches!(
            self.severity_level,
            SeverityLevel::Error
                | SeverityLevel::Error2
                | SeverityLevel::Error3
                | SeverityLevel::Error4
                | SeverityLevel::Fatal
                | SeverityLevel::Fatal2
                | SeverityLevel::Fatal3
                | SeverityLevel::Fatal4
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_log() -> Log {
        Log::new(
            1_000_000_000_000_000_000,
            SeverityLevel::Info,
            Some("INFO".to_string()),
            "Test log message".to_string(),
            Attributes::new(),
            Some("trace123".to_string()),
            Some("span456".to_string()),
            Some("test-service".to_string()),
        )
    }

    #[test]
    fn test_log_creation() {
        let log = create_test_log();
        assert_eq!(log.severity_level, SeverityLevel::Info);
        assert_eq!(log.body, "Test log message");
        assert_eq!(log.trace_id, Some("trace123".to_string()));
        assert_eq!(log.span_id, Some("span456".to_string()));
        assert_eq!(log.service_name, Some("test-service".to_string()));
    }

    #[test]
    fn test_log_is_correlated() {
        let mut log = create_test_log();
        assert!(log.is_correlated());

        log.trace_id = None;
        assert!(!log.is_correlated());

        log.trace_id = Some("trace".to_string());
        log.span_id = None;
        assert!(!log.is_correlated());
    }

    #[test]
    fn test_log_is_error() {
        let mut log = create_test_log();
        assert!(!log.is_error());

        log.severity_level = SeverityLevel::Error;
        assert!(log.is_error());

        log.severity_level = SeverityLevel::Fatal;
        assert!(log.is_error());

        log.severity_level = SeverityLevel::Warn;
        assert!(!log.is_error());
    }

    #[test]
    fn test_severity_level_ordering() {
        assert!(SeverityLevel::Error > SeverityLevel::Warn);
        assert!(SeverityLevel::Warn > SeverityLevel::Info);
        assert!(SeverityLevel::Info > SeverityLevel::Debug);
        assert!(SeverityLevel::Debug > SeverityLevel::Trace);
    }

    #[test]
    fn test_severity_level_as_str() {
        assert_eq!(SeverityLevel::Info.as_str(), "INFO");
        assert_eq!(SeverityLevel::Info2.as_str(), "INFO");
        assert_eq!(SeverityLevel::Error.as_str(), "ERROR");
        assert_eq!(SeverityLevel::Fatal.as_str(), "FATAL");
    }

    #[test]
    fn test_log_serde() {
        let log = create_test_log();
        let json = serde_json::to_string(&log).unwrap();
        let deserialized: Log = serde_json::from_str(&json).unwrap();
        assert_eq!(log, deserialized);
    }

    #[test]
    fn test_log_with_attributes() {
        let mut attrs = Attributes::new();
        attrs.insert("user_id", "12345");
        attrs.insert("request_id", "abc-def");
        let log = Log::new(
            1_000_000_000,
            SeverityLevel::Info,
            None,
            "User logged in".to_string(),
            attrs.clone(),
            None,
            None,
            Some("auth-service".to_string()),
        );
        assert_eq!(log.attributes.get_string("user_id"), Some("12345"));
        assert_eq!(log.attributes.get_string("request_id"), Some("abc-def"));
    }
}
