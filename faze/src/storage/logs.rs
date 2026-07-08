//! Log persistence: inserts and listing.

use super::rows::{attrs_to_json, log_from_row, opt_to_json, to_json};
use super::{Result, Storage, service_filtered_query};
use crate::models::Log;
use rusqlite::{Result as SqliteResult, params};

impl Storage {
    /// Insert a log
    pub fn insert_log(&self, log: &Log) -> Result<()> {
        self.insert_logs(std::slice::from_ref(log))
    }

    /// Insert multiple logs atomically under a single transaction.
    pub fn insert_logs(&self, logs: &[Log]) -> Result<()> {
        if logs.is_empty() {
            return Ok(());
        }
        self.with_tx(|tx| {
            let mut stmt = tx.prepare(
                "INSERT INTO logs (
                    time_unix_nano, severity_level, severity_text, body,
                    attributes, trace_id, span_id, service_name,
                    observed_time_unix_nano, event_name, flags,
                    resource_attributes, scope
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            )?;
            for log in logs {
                let attributes_json = to_json(&log.attributes)?;
                let resource_attributes_json = attrs_to_json(&log.resource_attributes)?;
                let scope_json = opt_to_json(log.scope.as_ref())?;
                stmt.execute(params![
                    log.time_unix_nano,
                    log.severity_level.as_db_str(),
                    &log.severity_text,
                    &log.body,
                    attributes_json,
                    &log.trace_id,
                    &log.span_id,
                    &log.service_name,
                    log.observed_time_unix_nano,
                    &log.event_name,
                    log.flags,
                    resource_attributes_json,
                    scope_json,
                ])?;
            }
            Ok(())
        })
    }

    /// List logs with optional filters
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_logs(&self, service_name: Option<&str>, limit: Option<usize>) -> Result<Vec<Log>> {
        let (query, params_vec) = service_filtered_query(
            "SELECT time_unix_nano, severity_level, severity_text, body,
                    attributes, trace_id, span_id, service_name,
                    observed_time_unix_nano, event_name, flags,
                    resource_attributes, scope
               FROM logs",
            "time_unix_nano",
            service_name,
            limit,
        );

        let conn = self.lock()?;
        let mut stmt = conn.prepare(&query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(AsRef::as_ref).collect();

        let logs = stmt
            .query_map(&params_refs[..], log_from_row)?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(logs)
    }

    /// Get all logs correlated with a trace, oldest first
    #[allow(clippy::significant_drop_tightening)]
    pub fn get_logs_by_trace_id(&self, trace_id: &str) -> Result<Vec<Log>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT time_unix_nano, severity_level, severity_text, body,
                    attributes, trace_id, span_id, service_name,
                    observed_time_unix_nano, event_name, flags,
                    resource_attributes, scope
               FROM logs
              WHERE trace_id = ?1
              ORDER BY time_unix_nano",
        )?;

        let logs = stmt
            .query_map([trace_id], log_from_row)?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(logs)
    }

    /// Get count of logs
    pub fn count_logs(&self) -> Result<i64> {
        self.count_rows("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Attributes, SeverityLevel};

    #[test]
    fn test_insert_and_list_logs() {
        let storage = Storage::new_in_memory().unwrap();
        let log = Log::new(
            1_000_000_000,
            SeverityLevel::Info,
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
    fn test_log_full_fidelity_roundtrip() {
        use crate::models::InstrumentationScope;

        let storage = Storage::new_in_memory().unwrap();
        let mut resource_attrs = Attributes::new();
        resource_attrs.insert("host.name", "box-1");

        let log = Log::new(
            1_000_000_000,
            SeverityLevel::Warn,
            Some("warning".to_string()),
            "disk almost full".to_string(),
            Attributes::new(),
            Some("trace1".to_string()),
            Some("span1".to_string()),
            Some("test-service".to_string()),
        )
        .with_observed_time(Some(1_000_000_001))
        .with_event_name(Some("disk.check".to_string()))
        .with_flags(Some(1))
        .with_resource_attributes(resource_attrs)
        .with_scope(Some(InstrumentationScope::new(
            "test-lib".to_string(),
            None,
            Attributes::new(),
        )));

        storage.insert_log(&log).unwrap();
        let logs = storage.list_logs(None, None).unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], log);
    }

    #[test]
    fn test_get_logs_by_trace_id() {
        let storage = Storage::new_in_memory().unwrap();
        let make_log = |time: i64, body: &str, trace_id: Option<&str>| {
            Log::new(
                time,
                SeverityLevel::Info,
                None,
                body.to_string(),
                Attributes::new(),
                trace_id.map(String::from),
                None,
                Some("test-service".to_string()),
            )
        };

        storage
            .insert_logs(&[
                make_log(3_000, "late", Some("trace1")),
                make_log(1_000, "early", Some("trace1")),
                make_log(2_000, "other trace", Some("trace2")),
                make_log(4_000, "uncorrelated", None),
            ])
            .unwrap();

        let logs = storage.get_logs_by_trace_id("trace1").unwrap();

        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].body, "early");
        assert_eq!(logs[1].body, "late");

        assert!(storage.get_logs_by_trace_id("missing").unwrap().is_empty());
    }
}
