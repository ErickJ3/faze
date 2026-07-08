-- v0 -> v1: full OTLP fidelity columns.
-- Runs only on pre-versioned databases (user_version = 0 with existing tables);
-- fresh databases get the full CREATE TABLE schema instead.
ALTER TABLE spans ADD COLUMN events TEXT;
ALTER TABLE spans ADD COLUMN links TEXT;
ALTER TABLE spans ADD COLUMN trace_state TEXT;
ALTER TABLE spans ADD COLUMN dropped_attributes_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE spans ADD COLUMN dropped_events_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE spans ADD COLUMN dropped_links_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE spans ADD COLUMN resource_attributes TEXT;
ALTER TABLE spans ADD COLUMN scope TEXT;

ALTER TABLE logs ADD COLUMN observed_time_unix_nano INTEGER;
ALTER TABLE logs ADD COLUMN event_name TEXT;
ALTER TABLE logs ADD COLUMN flags INTEGER;
ALTER TABLE logs ADD COLUMN resource_attributes TEXT;
ALTER TABLE logs ADD COLUMN scope TEXT;

ALTER TABLE metrics ADD COLUMN is_monotonic INTEGER;
ALTER TABLE metrics ADD COLUMN distribution TEXT;
ALTER TABLE metrics ADD COLUMN exemplars TEXT;
ALTER TABLE metrics ADD COLUMN resource_attributes TEXT;
ALTER TABLE metrics ADD COLUMN scope TEXT;
