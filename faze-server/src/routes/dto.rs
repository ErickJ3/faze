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
    /// Return only logs correlated with this trace (ignores other filters)
    pub trace_id: Option<String>,
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

/// Response for the aggregate stats endpoint
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    /// Total number of stored spans.
    pub spans: i64,
    /// Total number of stored logs.
    pub logs: i64,
    /// Total number of stored metrics.
    pub metrics: i64,
    /// Global trace aggregates.
    pub traces: TraceStatsDto,
    /// Per-service trace counts, most active first.
    pub services: Vec<ServiceStatDto>,
    /// Time-bucketed trace activity, oldest bucket first.
    pub activity: Vec<TimeBucketDto>,
}

/// Global trace aggregates.
#[derive(Debug, Serialize)]
pub struct TraceStatsDto {
    /// Total number of distinct traces.
    pub total: i64,
    /// Number of traces containing at least one error span.
    pub errors: i64,
    /// Mean trace duration in milliseconds.
    pub avg_duration_ms: f64,
}

impl From<faze::storage::TraceStats> for TraceStatsDto {
    fn from(stats: faze::storage::TraceStats) -> Self {
        Self {
            total: stats.total,
            errors: stats.errors,
            avg_duration_ms: stats.avg_duration_ms,
        }
    }
}

/// Per-service trace counts.
#[derive(Debug, Serialize)]
pub struct ServiceStatDto {
    /// Service name.
    pub name: String,
    /// Number of traces attributed to the service.
    pub trace_count: i64,
    /// Number of those traces containing at least one error span.
    pub error_count: i64,
}

impl From<faze::storage::ServiceStat> for ServiceStatDto {
    fn from(stat: faze::storage::ServiceStat) -> Self {
        Self {
            name: stat.name,
            trace_count: stat.trace_count,
            error_count: stat.error_count,
        }
    }
}

/// One time bucket of trace activity.
#[derive(Debug, Serialize)]
pub struct TimeBucketDto {
    /// Bucket start as nanoseconds since the Unix epoch.
    pub bucket_start_unix_nano: i64,
    /// Number of traces starting in this bucket.
    pub total: i64,
    /// Number of those traces containing at least one error span.
    pub errors: i64,
}

impl From<faze::storage::TraceTimeBucket> for TimeBucketDto {
    fn from(bucket: faze::storage::TraceTimeBucket) -> Self {
        Self {
            bucket_start_unix_nano: bucket.bucket_start_unix_nano,
            total: bucket.total,
            errors: bucket.errors,
        }
    }
}
