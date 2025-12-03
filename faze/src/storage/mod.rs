mod convert;
mod db_path;
mod schema;

use crate::models::{
    AggregationTemporality, Attributes, Log, Metric, MetricDataPoint, MetricType, Span, Trace,
};
use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use convert::{
    from_json, parse_metric_type, parse_severity_level, parse_temporality, span_from_row, to_json,
};
pub use db_path::{
    detect_project_root, get_config_dir, get_data_dir, get_default_db_path, get_project_db_path,
};
use schema::init_schema;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Main storage interface for Faze
///
/// By default, Faze stores data in a file-based database (`faze.db`) to prevent
/// excessive memory usage for large projects. You can:
/// - Use the default database: `Storage::new()`
/// - For testing and debug, use: `Storage::new_in_memory()`
/// - Specify a custom path: `Storage::new_with_path("custom.db")`
/// - Delete the database: `Storage::delete_database("faze.db")`
#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    /// Create a new storage instance with automatic project-based database
    ///
    /// This will:
    /// 1. Detect the current project by looking for markers (.git, Cargo.toml, package.json, etc.)
    /// 2. Create a database in ~/.local/share/faze/<project_name>.db
    /// 3. Multiple terminals in the same project will share the same database
    pub fn new() -> Result<Self> {
        let db_path = get_project_db_path().map_err(|e| {
            StorageError::InvalidInput(format!("Failed to determine database path: {}", e))
        })?;

        Self::new_with_path(&db_path)
    }

    /// Create a new storage instance with an in-memory database (only for testing, no use this in app pls!)
    ///
    /// This is available in test mode for all crates to use
    #[doc(hidden)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create a new storage instance with a custom file path
    pub fn new_with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();

        if let Some(parent) = path_ref.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::InvalidInput(format!("Failed to create directory: {}", e))
            })?;
        }

        let conn = Connection::open(path)?;
        init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Delete the database file
    pub fn delete_database<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        std::fs::remove_file(path)
    }

    /// Insert a span
    pub fn insert_span(&self, span: &Span) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let attributes_json = to_json(&span.attributes)?;
        let status_json = to_json(&span.status)?;

        conn.execute(
            "INSERT INTO spans (
                span_id, trace_id, parent_span_id, name, kind,
                start_time_unix_nano, end_time_unix_nano,
                attributes, status, service_name
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &span.span_id,
                &span.trace_id,
                &span.parent_span_id,
                &span.name,
                format!("{:?}", span.kind),
                span.start_time_unix_nano,
                span.end_time_unix_nano,
                attributes_json,
                status_json,
                &span.service_name,
            ],
        )?;

        Ok(())
    }

    /// Insert multiple spans
    pub fn insert_spans(&self, spans: &[Span]) -> Result<()> {
        for span in spans {
            self.insert_span(span)?;
        }
        Ok(())
    }

    /// Insert a log
    pub fn insert_log(&self, log: &Log) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let attributes_json = to_json(&log.attributes)?;

        conn.execute(
            "INSERT INTO logs (
                time_unix_nano, severity_level, severity_text, body,
                attributes, trace_id, span_id, service_name
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                log.time_unix_nano,
                format!("{:?}", log.severity_level),
                &log.severity_text,
                &log.body,
                attributes_json,
                &log.trace_id,
                &log.span_id,
                &log.service_name,
            ],
        )?;

        Ok(())
    }

    /// Insert multiple logs
    pub fn insert_logs(&self, logs: &[Log]) -> Result<()> {
        for log in logs {
            self.insert_log(log)?;
        }
        Ok(())
    }

    /// Insert a metric
    pub fn insert_metric(&self, metric: &Metric) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        for data_point in &metric.data_points {
            let attributes_json = to_json(&data_point.attributes)?;

            conn.execute(
                "INSERT INTO metrics (
                    name, description, unit, metric_type, temporality,
                    time_unix_nano, start_time_unix_nano, value,
                    attributes, service_name
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    &metric.name,
                    &metric.description,
                    &metric.unit,
                    format!("{:?}", metric.metric_type),
                    format!("{:?}", metric.temporality),
                    data_point.time_unix_nano,
                    data_point.start_time_unix_nano,
                    data_point.value,
                    attributes_json,
                    &metric.service_name,
                ],
            )?;
        }

        Ok(())
    }

    /// Insert multiple metrics
    pub fn insert_metrics(&self, metrics: &[Metric]) -> Result<()> {
        for metric in metrics {
            self.insert_metric(metric)?;
        }
        Ok(())
    }

    /// Get a complete trace by ID
    pub fn get_trace_by_id(&self, trace_id: &str) -> Result<Trace> {
        let spans = self.get_spans_by_trace_id(trace_id)?;

        if spans.is_empty() {
            return Err(StorageError::NotFound(format!(
                "Trace not found: {}",
                trace_id
            )));
        }

        Ok(Trace::new(trace_id.to_string(), spans))
    }

    /// Get all spans for a trace
    fn get_spans_by_trace_id(&self, trace_id: &str) -> Result<Vec<Span>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT span_id, trace_id, parent_span_id, name, kind,
                    start_time_unix_nano, end_time_unix_nano,
                    attributes, status, service_name
             FROM spans
             WHERE trace_id = ?1
             ORDER BY start_time_unix_nano",
        )?;

        let spans = stmt
            .query_map([trace_id], span_from_row)?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(spans)
    }

    /// List traces with optional filters
    pub fn list_traces(
        &self,
        service_name: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Trace>> {
        let conn = self.conn.lock().unwrap();

        let (query, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(service) =
            service_name
        {
            (
                "SELECT DISTINCT trace_id FROM spans WHERE service_name = ?1 ORDER BY start_time_unix_nano DESC LIMIT ?2".to_string(),
                vec![Box::new(service.to_string()), Box::new(limit.unwrap_or(100) as i64)],
            )
        } else {
            (
                "SELECT DISTINCT trace_id FROM spans ORDER BY start_time_unix_nano DESC LIMIT ?1"
                    .to_string(),
                vec![Box::new(limit.unwrap_or(100) as i64)],
            )
        };

        let mut stmt = conn.prepare(&query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let trace_ids: Vec<String> = stmt
            .query_map(&params_refs[..], |row| row.get(0))?
            .collect::<SqliteResult<Vec<_>>>()?;

        drop(stmt);
        drop(conn);

        let mut traces = Vec::new();
        for trace_id in trace_ids {
            if let Ok(trace) = self.get_trace_by_id(&trace_id) {
                traces.push(trace);
            }
        }

        Ok(traces)
    }

    /// List logs with optional filters
    pub fn list_logs(&self, service_name: Option<&str>, limit: Option<usize>) -> Result<Vec<Log>> {
        let conn = self.conn.lock().unwrap();

        let (query, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) =
            if let Some(service) = service_name {
                (
                    "SELECT time_unix_nano, severity_level, severity_text, body,
                        attributes, trace_id, span_id, service_name
                 FROM logs
                 WHERE service_name = ?1
                 ORDER BY time_unix_nano DESC
                 LIMIT ?2"
                        .to_string(),
                    vec![
                        Box::new(service.to_string()),
                        Box::new(limit.unwrap_or(100) as i64),
                    ],
                )
            } else {
                (
                    "SELECT time_unix_nano, severity_level, severity_text, body,
                        attributes, trace_id, span_id, service_name
                 FROM logs
                 ORDER BY time_unix_nano DESC
                 LIMIT ?1"
                        .to_string(),
                    vec![Box::new(limit.unwrap_or(100) as i64)],
                )
            };

        let mut stmt = conn.prepare(&query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let logs = stmt
            .query_map(&params_refs[..], |row| {
                let attributes_json: String = row.get(4)?;
                let attributes = from_json(&attributes_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                let severity_str: String = row.get(1)?;
                let severity_level = parse_severity_level(&severity_str);

                Ok(Log::new(
                    row.get(0)?,
                    severity_level,
                    row.get(2)?,
                    row.get(3)?,
                    attributes,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(logs)
    }

    pub fn list_metrics(
        &self,
        service_name: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Metric>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::InvalidInput(format!("Mutex poisoned: {}", e)))?;
        let limit_value = limit.unwrap_or(100) as i64;

        let (query, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) =
            if let Some(service) = service_name {
                (
                    "SELECT name, description, unit, metric_type, temporality,
                            time_unix_nano, start_time_unix_nano, value,
                            attributes, service_name
                       FROM metrics
                       WHERE service_name = ?1
                       ORDER BY time_unix_nano DESC
                       LIMIT ?2"
                        .to_string(),
                    vec![Box::new(service.to_string()), Box::new(limit_value)],
                )
            } else {
                (
                    "SELECT name, description, unit, metric_type, temporality,
                            time_unix_nano, start_time_unix_nano, value,
                            attributes, service_name
                       FROM metrics
                       ORDER BY time_unix_nano DESC
                       LIMIT ?1"
                        .to_string(),
                    vec![Box::new(limit_value)],
                )
            };

        let mut stmt = conn.prepare(&query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let metrics = stmt
            .query_map(&params_refs[..], |row| {
                let attributes_json: String = row.get(8)?;

                let raw_json: serde_json::Value =
                    serde_json::from_str(&attributes_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            8,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                let mut attributes = Attributes::default();
                if let serde_json::Value::Object(map) = raw_json {
                    for (k, v) in map {
                        attributes.insert(k, v.to_string());
                    }
                }

                let metric_type_str: String = row.get(3)?;
                let metric_type: MetricType = parse_metric_type(&metric_type_str);

                let temporality_str: String = row.get(4)?;
                let temporality: AggregationTemporality = parse_temporality(&temporality_str);

                let data_point = MetricDataPoint {
                    time_unix_nano: row.get(5)?,
                    start_time_unix_nano: row.get(6)?,
                    value: row.get(7)?,
                    attributes,
                };

                Ok(Metric {
                    name: row.get(0)?,
                    description: row.get(1)?,
                    unit: row.get(2)?,
                    metric_type,
                    temporality,
                    data_points: vec![data_point],
                    service_name: row.get(9)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<Metric>>>()
            .map_err(StorageError::from)?;

        Ok(metrics)
    }

    /// Get count of spans
    pub fn count_spans(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM spans", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get count of logs
    pub fn count_logs(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM logs", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get count of metrics
    pub fn count_metrics(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM metrics", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Attributes, SpanKind, Status};

    fn create_test_span(span_id: &str, trace_id: &str) -> Span {
        Span::new(
            span_id.to_string(),
            trace_id.to_string(),
            None,
            "test-operation".to_string(),
            SpanKind::Server,
            1_000_000_000_000_000_000,
            1_000_000_000_100_000_000,
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        )
    }

    #[test]
    fn test_storage_new_in_memory() {
        let storage = Storage::new_in_memory();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_insert_and_get_span() {
        let storage = Storage::new_in_memory().unwrap();
        let span = create_test_span("span1", "trace1");

        storage.insert_span(&span).unwrap();
        let trace = storage.get_trace_by_id("trace1").unwrap();

        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.spans[0].span_id, "span1");
    }

    #[test]
    fn test_insert_multiple_spans() {
        let storage = Storage::new_in_memory().unwrap();
        let spans = vec![
            create_test_span("span1", "trace1"),
            create_test_span("span2", "trace1"),
        ];

        storage.insert_spans(&spans).unwrap();
        let trace = storage.get_trace_by_id("trace1").unwrap();

        assert_eq!(trace.spans.len(), 2);
    }

    #[test]
    fn test_list_traces() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .insert_span(&create_test_span("span1", "trace1"))
            .unwrap();
        storage
            .insert_span(&create_test_span("span2", "trace2"))
            .unwrap();

        let traces = storage.list_traces(None, None).unwrap();
        assert_eq!(traces.len(), 2);
    }

    #[test]
    fn test_count_spans() {
        let storage = Storage::new_in_memory().unwrap();
        assert_eq!(storage.count_spans().unwrap(), 0);

        storage
            .insert_span(&create_test_span("span1", "trace1"))
            .unwrap();
        assert_eq!(storage.count_spans().unwrap(), 1);

        storage
            .insert_span(&create_test_span("span2", "trace1"))
            .unwrap();
        assert_eq!(storage.count_spans().unwrap(), 2);
    }

    #[test]
    fn test_insert_and_list_logs() {
        let storage = Storage::new_in_memory().unwrap();
        let log = Log::new(
            1_000_000_000,
            crate::models::SeverityLevel::Info,
            Some("INFO".to_string()),
            "Test log".to_string(),
            Attributes::new(),
            None,
            None,
            Some("test-service".to_string()),
        );

        storage.insert_log(&log).unwrap();
        let logs = storage.list_logs(None, None).unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].body, "Test log");
    }

    #[test]
    fn test_get_nonexistent_trace() {
        let storage = Storage::new_in_memory().unwrap();
        let result = storage.get_trace_by_id("nonexistent");
        assert!(result.is_err());
    }
}
