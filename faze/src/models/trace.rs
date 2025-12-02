use super::span::Span;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a complete trace (collection of related spans)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    /// Unique trace identifier
    pub trace_id: String,
    /// All spans in this trace
    pub spans: Vec<Span>,
    /// Service name (from root span)
    pub service_name: Option<String>,
}

impl Trace {
    pub fn new(trace_id: String, spans: Vec<Span>) -> Self {
        let service_name = spans
            .iter()
            .find(|s| s.is_root())
            .and_then(|s| s.service_name.clone())
            .or_else(|| spans.first().and_then(|s| s.service_name.clone()));

        Self {
            trace_id,
            spans,
            service_name,
        }
    }

    /// Get the root span
    pub fn root_span(&self) -> Option<&Span> {
        self.spans.iter().find(|s| s.is_root())
    }

    /// Get total duration of the trace
    pub fn duration_nanos(&self) -> i64 {
        if self.spans.is_empty() {
            return 0;
        }

        let min_start = self
            .spans
            .iter()
            .map(|s| s.start_time_unix_nano)
            .min()
            .unwrap_or(0);

        let max_end = self
            .spans
            .iter()
            .map(|s| s.end_time_unix_nano)
            .max()
            .unwrap_or(0);

        max_end - min_start
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        self.duration_nanos() as f64 / 1_000_000.0
    }

    /// Get start time
    pub fn start_time(&self) -> Option<DateTime<Utc>> {
        self.spans
            .iter()
            .map(|s| s.start_time_unix_nano)
            .min()
            .map(DateTime::from_timestamp_nanos)
    }

    /// Get end time
    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        self.spans
            .iter()
            .map(|s| s.end_time_unix_nano)
            .max()
            .map(DateTime::from_timestamp_nanos)
    }

    /// Get number of spans
    pub fn span_count(&self) -> usize {
        self.spans.len()
    }

    /// Check if trace has any error spans
    pub fn has_errors(&self) -> bool {
        self.spans.iter().any(|s| s.is_error())
    }

    /// Get all error spans
    pub fn error_spans(&self) -> Vec<&Span> {
        self.spans.iter().filter(|s| s.is_error()).collect()
    }

    /// Get spans by parent ID
    pub fn children_of(&self, parent_span_id: &str) -> Vec<&Span> {
        self.spans
            .iter()
            .filter(|s| {
                s.parent_span_id
                    .as_ref()
                    .map(|id| id == parent_span_id)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get span by ID
    pub fn get_span(&self, span_id: &str) -> Option<&Span> {
        self.spans.iter().find(|s| s.span_id == span_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        attributes::Attributes,
        span::{SpanKind, Status},
    };

    fn create_test_span(
        span_id: &str,
        parent_id: Option<&str>,
        start: i64,
        end: i64,
        is_error: bool,
    ) -> Span {
        let status = if is_error {
            Status::error("error")
        } else {
            Status::ok()
        };

        Span::new(
            span_id.to_string(),
            "trace123".to_string(),
            parent_id.map(|s| s.to_string()),
            format!("operation-{}", span_id),
            SpanKind::Server,
            start,
            end,
            Attributes::new(),
            status,
            Some("test-service".to_string()),
        )
    }

    #[test]
    fn test_trace_creation() {
        let spans = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 4000, false),
        ];

        let trace = Trace::new("trace123".to_string(), spans);
        assert_eq!(trace.trace_id, "trace123");
        assert_eq!(trace.span_count(), 2);
        assert_eq!(trace.service_name, Some("test-service".to_string()));
    }

    #[test]
    fn test_trace_root_span() {
        let spans = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 4000, false),
            create_test_span("span3", Some("span1"), 2500, 3500, false),
        ];

        let trace = Trace::new("trace123".to_string(), spans);
        let root = trace.root_span().unwrap();
        assert_eq!(root.span_id, "span1");
        assert!(root.is_root());
    }

    #[test]
    fn test_trace_duration() {
        let spans = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 4000, false),
        ];

        let trace = Trace::new("trace123".to_string(), spans);
        assert_eq!(trace.duration_nanos(), 4000); // 5000 - 1000
        assert_eq!(trace.duration_ms(), 0.004); // 4000 nanos = 0.004 ms
    }

    #[test]
    fn test_trace_has_errors() {
        let spans_ok = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 4000, false),
        ];

        let trace_ok = Trace::new("trace1".to_string(), spans_ok);
        assert!(!trace_ok.has_errors());

        let spans_error = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 4000, true),
        ];

        let trace_error = Trace::new("trace2".to_string(), spans_error);
        assert!(trace_error.has_errors());
    }

    #[test]
    fn test_trace_error_spans() {
        let spans = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 3000, true),
            create_test_span("span3", Some("span1"), 3500, 4500, true),
        ];

        let trace = Trace::new("trace123".to_string(), spans);
        let errors = trace.error_spans();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].span_id, "span2");
        assert_eq!(errors[1].span_id, "span3");
    }

    #[test]
    fn test_trace_children_of() {
        let spans = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 3000, false),
            create_test_span("span3", Some("span1"), 3500, 4500, false),
            create_test_span("span4", Some("span2"), 2500, 2800, false),
        ];

        let trace = Trace::new("trace123".to_string(), spans);

        let children_of_span1 = trace.children_of("span1");
        assert_eq!(children_of_span1.len(), 2);

        let children_of_span2 = trace.children_of("span2");
        assert_eq!(children_of_span2.len(), 1);
        assert_eq!(children_of_span2[0].span_id, "span4");
    }

    #[test]
    fn test_trace_get_span() {
        let spans = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 4000, false),
        ];

        let trace = Trace::new("trace123".to_string(), spans);

        let span = trace.get_span("span2").unwrap();
        assert_eq!(span.span_id, "span2");

        assert!(trace.get_span("nonexistent").is_none());
    }

    #[test]
    fn test_trace_empty() {
        let trace = Trace::new("trace123".to_string(), vec![]);
        assert_eq!(trace.span_count(), 0);
        assert_eq!(trace.duration_nanos(), 0);
        assert!(trace.root_span().is_none());
        assert!(!trace.has_errors());
    }

    #[test]
    fn test_trace_serde() {
        let spans = vec![
            create_test_span("span1", None, 1000, 5000, false),
            create_test_span("span2", Some("span1"), 2000, 4000, false),
        ];

        let trace = Trace::new("trace123".to_string(), spans);

        let json = serde_json::to_string(&trace).unwrap();
        let deserialized: Trace = serde_json::from_str(&json).unwrap();
        assert_eq!(trace, deserialized);
    }
}
