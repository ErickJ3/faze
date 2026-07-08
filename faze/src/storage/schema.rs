use rusqlite::Connection;

const SPANS_SCHEMA: &str = include_str!("sql/spans.sql");
const LOGS_SCHEMA: &str = include_str!("sql/logs.sql");
const METRICS_SCHEMA: &str = include_str!("sql/metrics.sql");
const MIGRATION_V1: &str = include_str!("sql/migrations/v1.sql");

/// Current on-disk schema version, tracked via `PRAGMA user_version`.
const SCHEMA_VERSION: i64 = 1;

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version == 0 {
        // Pre-versioned databases already have the v0 tables; ALTER them in
        // place. Fresh databases get the full schema directly.
        let legacy = table_exists(conn, "spans")?;
        conn.execute_batch("BEGIN")?;
        let result = (|| {
            if legacy {
                conn.execute_batch(MIGRATION_V1)?;
            } else {
                create_tables(conn)?;
            }
            conn.pragma_update(None, "user_version", SCHEMA_VERSION)
        })();
        match result {
            Ok(()) => conn.execute_batch("COMMIT")?,
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }
    }

    // Idempotent for already-migrated databases (CREATE IF NOT EXISTS no-ops).
    create_tables(conn)
}

fn create_tables(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SPANS_SCHEMA)?;
    conn.execute_batch(LOGS_SCHEMA)?;
    conn.execute_batch(METRICS_SCHEMA)?;
    Ok(())
}

fn table_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [name],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_schema() {
        let conn = Connection::open_in_memory().unwrap();
        let result = init_schema(&conn);
        assert!(result.is_ok());

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();

        assert!(tables.contains(&"spans".to_string()));
        assert!(tables.contains(&"logs".to_string()));
        assert!(tables.contains(&"metrics".to_string()));
    }

    #[test]
    fn test_init_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_schema(&conn).is_ok());
        assert!(init_schema(&conn).is_ok());
        assert!(init_schema(&conn).is_ok());
    }

    #[test]
    fn test_init_schema_sets_user_version() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    /// The v0 DDL as shipped before schema versioning, frozen for migration tests.
    const LEGACY_V0_DDL: &str = "
        CREATE TABLE spans (
            span_id TEXT NOT NULL,
            trace_id TEXT NOT NULL,
            parent_span_id TEXT,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            start_time_unix_nano INTEGER NOT NULL,
            end_time_unix_nano INTEGER NOT NULL,
            attributes TEXT NOT NULL,
            status TEXT NOT NULL,
            service_name TEXT,
            PRIMARY KEY (span_id, trace_id)
        );
        CREATE TABLE logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            time_unix_nano INTEGER NOT NULL,
            severity_level TEXT NOT NULL,
            severity_text TEXT,
            body TEXT NOT NULL,
            attributes TEXT NOT NULL,
            trace_id TEXT,
            span_id TEXT,
            service_name TEXT
        );
        CREATE TABLE metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            unit TEXT,
            metric_type TEXT NOT NULL,
            temporality TEXT NOT NULL,
            time_unix_nano INTEGER NOT NULL,
            start_time_unix_nano INTEGER,
            value REAL NOT NULL,
            attributes TEXT NOT NULL,
            service_name TEXT
        );
    ";

    fn open_legacy_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(LEGACY_V0_DDL).unwrap();
        conn.execute_batch(
            "INSERT INTO spans VALUES
                ('s1', 't1', NULL, 'op', 'Server', 1, 2, '{}', '{\"code\":\"OK\",\"message\":null}', 'svc');
             INSERT INTO logs (time_unix_nano, severity_level, severity_text, body, attributes, trace_id, span_id, service_name)
                VALUES (1, 'Info', 'info', 'msg', '{}', NULL, NULL, 'svc');
             INSERT INTO metrics (name, description, unit, metric_type, temporality, time_unix_nano, start_time_unix_nano, value, attributes, service_name)
                VALUES ('m', NULL, NULL, 'Gauge', 'Unspecified', 1, NULL, 2.0, '{}', 'svc');",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_migrate_legacy_v0_database() {
        let conn = open_legacy_db();
        init_schema(&conn).unwrap();

        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        // New columns exist and legacy rows read back with defaults.
        let (events, dropped): (Option<String>, i64) = conn
            .query_row(
                "SELECT events, dropped_attributes_count FROM spans WHERE span_id = 's1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(events, None);
        assert_eq!(dropped, 0);

        let observed: Option<i64> = conn
            .query_row("SELECT observed_time_unix_nano FROM logs", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(observed, None);

        let distribution: Option<String> = conn
            .query_row("SELECT distribution FROM metrics", [], |row| row.get(0))
            .unwrap();
        assert_eq!(distribution, None);

        // Migration must be idempotent through the version gate.
        init_schema(&conn).unwrap();
    }
}
