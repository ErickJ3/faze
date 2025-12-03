use rusqlite::Connection;

const SPANS_SCHEMA: &str = include_str!("sql/spans.sql");
const LOGS_SCHEMA: &str = include_str!("sql/logs.sql");
const METRICS_SCHEMA: &str = include_str!("sql/metrics.sql");

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SPANS_SCHEMA)?;
    conn.execute_batch(LOGS_SCHEMA)?;
    conn.execute_batch(METRICS_SCHEMA)?;

    Ok(())
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
}
