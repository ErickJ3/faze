pub mod models;
pub mod storage;

// Re-exports
pub use models::{
    AttributeValue, Attributes, Log, Metric, MetricDataPoint, MetricType, Resource, SeverityLevel,
    Span, SpanKind, Status, StatusCode, Trace,
};
pub use storage::{
    Storage, StorageError, detect_project_root, get_config_dir, get_data_dir, get_default_db_path,
    get_project_db_path,
};
