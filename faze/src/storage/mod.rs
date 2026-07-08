//! SQLite-backed storage layer for spans, logs, and metrics.

mod db_path;
mod logs;
mod metrics;
mod rows;
mod schema;
mod spans;
mod stats;

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use thiserror::Error;

pub use db_path::{detect_project_root, get_data_dir, get_project_db_path};
pub use stats::{ServiceStat, TraceStats, TraceTimeBucket};
use schema::init_schema;

/// Errors returned by [`Storage`] operations.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Underlying `SQLite` failure.
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// JSON (de)serialization failure for attributes or status.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Requested entity does not exist.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Caller supplied invalid input or environment state.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// The shared connection mutex was poisoned by a panic in another thread.
    #[error("Storage lock poisoned")]
    LockPoisoned,
}

/// Specialised `Result` for [`Storage`] operations.
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
    /// 2. Create a database in `~/.local/share/faze/<project_name>.db`
    /// 3. Multiple terminals in the same project will share the same database
    pub fn new() -> Result<Self> {
        let db_path = get_project_db_path().map_err(|e| {
            StorageError::InvalidInput(format!("Failed to determine database path: {e}"))
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
                StorageError::InvalidInput(format!("Failed to create directory: {e}"))
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

    fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| StorageError::LockPoisoned)
    }

    /// Run `f` inside a single transaction, committing on success.
    #[allow(clippy::significant_drop_tightening)]
    fn with_tx<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> Result<()>,
    {
        let mut conn = self.lock()?;
        let tx = conn.transaction()?;
        f(&tx)?;
        tx.commit()?;
        Ok(())
    }

    /// Count all rows of a table. `table` must be a compile-time table name.
    #[allow(clippy::significant_drop_tightening)]
    fn count_rows(&self, table: &str) -> Result<i64> {
        let conn = self.lock()?;
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })?;
        Ok(count)
    }
}

/// Build a `SELECT` with an optional service-name filter, plus ordering and a
/// row cap. `base` and `order_col` come from compile-time literals only.
fn service_filtered_query(
    base: &str,
    order_col: &str,
    service_name: Option<&str>,
    limit: Option<usize>,
) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let limit_value = limit.map_or(100, |l| i64::try_from(l).unwrap_or(i64::MAX));

    service_name.map_or_else(
        || {
            (
                format!("{base} ORDER BY {order_col} DESC LIMIT ?1"),
                vec![Box::new(limit_value) as Box<dyn rusqlite::ToSql>],
            )
        },
        |service| {
            (
                format!("{base} WHERE service_name = ?1 ORDER BY {order_col} DESC LIMIT ?2"),
                vec![
                    Box::new(service.to_string()) as Box<dyn rusqlite::ToSql>,
                    Box::new(limit_value),
                ],
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_new_in_memory() {
        let storage = Storage::new_in_memory();
        assert!(storage.is_ok());
    }
}
