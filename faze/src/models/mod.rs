//! Domain types: spans, logs, metrics, traces, attributes, and resources.

/// Attribute key-value pairs attached to telemetry items.
pub mod attributes;
mod db_enum;
/// Log records emitted by services.
pub mod log;
/// Metrics with data points and aggregation semantics.
pub mod metric;
/// Resource descriptors identifying telemetry producers.
pub mod resource;
/// Spans representing units of work in a trace.
pub mod span;
/// Traces aggregating related spans.
pub mod trace;

// Re-exports
pub use attributes::{AttributeValue, Attributes};
pub use log::{Log, SeverityLevel};
pub use metric::{AggregationTemporality, Metric, MetricDataPoint, MetricType};
pub use resource::Resource;
pub use span::{Span, SpanKind, Status, StatusCode};
pub use trace::Trace;
