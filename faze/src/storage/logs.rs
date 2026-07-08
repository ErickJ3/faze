//! Log persistence: inserts and listing.

use super::rows::{log_from_row, to_json};
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
                    attributes, trace_id, span_id, service_name
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            for log in logs {
                let attributes_json = to_json(&log.attributes)?;
                stmt.execute(params![
                    log.time_unix_nano,
                    log.severity_level.as_db_str(),
                    &log.severity_text,
                    &log.body,
                    attributes_json,
                    &log.trace_id,
                    &log.span_id,
                    &log.service_name,
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
                    attributes, trace_id, span_id, service_name
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
}
