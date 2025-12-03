CREATE TABLE IF NOT EXISTS logs (
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

CREATE INDEX IF NOT EXISTS idx_logs_time ON logs(time_unix_nano);
CREATE INDEX IF NOT EXISTS idx_logs_trace_id ON logs(trace_id);
CREATE INDEX IF NOT EXISTS idx_logs_service_name ON logs(service_name);
CREATE INDEX IF NOT EXISTS idx_logs_severity ON logs(severity_level);
