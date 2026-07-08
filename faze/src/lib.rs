//! Faze core: domain models and SQLite-backed storage for spans, logs, and metrics.
//!
//! This crate is the substrate shared by the collector, server, and CLI.
//! It owns the on-disk schema and the typed APIs for inserting and querying
//! observability data.

pub mod models;
pub mod storage;

pub use models::{
    AttributeValue, Attributes, Log, Metric, MetricDataPoint, MetricType, Resource, SeverityLevel,
    Span, SpanKind, Status, StatusCode, Trace,
};
pub use storage::{Storage, StorageError, detect_project_root, get_data_dir, get_project_db_path};
