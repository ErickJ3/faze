//! Faze core: domain models and SQLite-backed storage for spans, logs, and metrics.
//!
//! This crate is the substrate shared by the collector, server, and CLI.
//! It owns the on-disk schema and the typed APIs for inserting and querying
//! observability data.

pub mod models;
pub mod storage;

pub use models::{
    AggregationTemporality, AttributeValue, Attributes, Distribution, Exemplar,
    InstrumentationScope, Log, Metric, MetricDataPoint, MetricType, QuantileValue, Resource,
    SeverityLevel, Span, SpanEvent, SpanKind, SpanLink, Status, StatusCode, Trace,
};
pub use storage::{Storage, StorageError, detect_project_root, get_data_dir, get_project_db_path};
