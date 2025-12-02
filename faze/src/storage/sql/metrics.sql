CREATE TABLE IF NOT EXISTS metrics (
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

CREATE INDEX IF NOT EXISTS idx_metrics_name ON metrics(name);
CREATE INDEX IF NOT EXISTS idx_metrics_time ON metrics(time_unix_nano);
CREATE INDEX IF NOT EXISTS idx_metrics_service_name ON metrics(service_name);
