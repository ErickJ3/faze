//! Request query parameters and response DTOs for the API.

use serde::{Deserialize, Serialize};

/// Query parameters for listing traces
#[derive(Debug, Deserialize)]
pub struct ListTracesQuery {
    /// Filter by service name
    pub service: Option<String>,
    /// Minimum duration in milliseconds
    pub min_duration: Option<f64>,
    /// Maximum duration in milliseconds
    pub max_duration: Option<f64>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Query parameters for listing logs
#[derive(Debug, Deserialize)]
pub struct ListLogsQuery {
    /// Filter by service name
    pub service: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

/// Query parameters for listing metrics
#[derive(Debug, Deserialize)]
pub struct ListMetricsQuery {
    /// Filter by service name
    pub service: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

/// Response for trace list
#[derive(Debug, Serialize)]
pub struct TraceListResponse {
    /// Trace summaries returned by the query.
    pub traces: Vec<TraceInfo>,
    /// Total count of traces in the response.
    pub total: usize,
}

/// Trace information for list view
#[derive(Debug, Serialize)]
pub struct TraceInfo {
    /// Trace identifier.
    pub trace_id: String,
    /// Service name associated with the root span, if known.
    pub service_name: Option<String>,
    /// Total trace duration in milliseconds.
    pub duration_ms: f64,
    /// Number of spans in the trace.
    pub span_count: usize,
    /// True when at least one span has error status.
    pub has_errors: bool,
    /// Trace start time as nanoseconds since the Unix epoch.
    pub start_time: Option<i64>,
    /// Operation name of the root span, if any.
    pub root_span_name: Option<String>,
    /// Kind of the root span, if any.
    pub root_span_kind: Option<faze::SpanKind>,
}

impl From<&faze::Trace> for TraceInfo {
    fn from(trace: &faze::Trace) -> Self {
        let root_span = trace.root_span();

        Self {
            trace_id: trace.trace_id.clone(),
            service_name: trace.service_name.clone(),
            duration_ms: trace.duration_ms(),
            span_count: trace.span_count(),
            has_errors: trace.has_errors(),
            start_time: trace.start_time().and_then(|dt| dt.timestamp_nanos_opt()),
            root_span_name: root_span.map(|s| s.name.clone()),
            root_span_kind: root_span.map(|s| s.kind),
        }
    }
}
