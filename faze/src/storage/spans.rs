//! Span and trace persistence: inserts, trace lookup, and listing.

use super::rows::{attrs_to_json, opt_to_json, slice_to_json, span_from_row, to_json};
use super::{Result, Storage, StorageError, service_filtered_query};
use crate::models::{Span, Trace};
use rusqlite::{Result as SqliteResult, params};

impl Storage {
    /// Insert a span
    pub fn insert_span(&self, span: &Span) -> Result<()> {
        self.insert_spans(std::slice::from_ref(span))
    }

    /// Insert multiple spans atomically under a single transaction.
    pub fn insert_spans(&self, spans: &[Span]) -> Result<()> {
        if spans.is_empty() {
            return Ok(());
        }
        self.with_tx(|tx| {
            let mut stmt = tx.prepare(
                "INSERT INTO spans (
                    span_id, trace_id, parent_span_id, name, kind,
                    start_time_unix_nano, end_time_unix_nano,
                    attributes, status, service_name,
                    events, links, trace_state,
                    dropped_attributes_count, dropped_events_count, dropped_links_count,
                    resource_attributes, scope
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            )?;
            for span in spans {
                let attributes_json = to_json(&span.attributes)?;
                let status_json = to_json(&span.status)?;
                let events_json = slice_to_json(&span.events)?;
                let links_json = slice_to_json(&span.links)?;
                let resource_attributes_json = attrs_to_json(&span.resource_attributes)?;
                let scope_json = opt_to_json(span.scope.as_ref())?;
                stmt.execute(params![
                    &span.span_id,
                    &span.trace_id,
                    &span.parent_span_id,
                    &span.name,
                    span.kind.as_db_str(),
                    span.start_time_unix_nano,
                    span.end_time_unix_nano,
                    attributes_json,
                    status_json,
                    &span.service_name,
                    events_json,
                    links_json,
                    &span.trace_state,
                    span.dropped_attributes_count,
                    span.dropped_events_count,
                    span.dropped_links_count,
                    resource_attributes_json,
                    scope_json,
                ])?;
            }
            Ok(())
        })
    }

    /// Get a complete trace by ID
    pub fn get_trace_by_id(&self, trace_id: &str) -> Result<Trace> {
        let spans = self.get_spans_by_trace_id(trace_id)?;

        if spans.is_empty() {
            return Err(StorageError::NotFound(format!(
                "Trace not found: {trace_id}"
            )));
        }

        Ok(Trace::new(trace_id.to_string(), spans))
    }

    /// Get all spans for a trace
    #[allow(clippy::significant_drop_tightening)]
    fn get_spans_by_trace_id(&self, trace_id: &str) -> Result<Vec<Span>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT span_id, trace_id, parent_span_id, name, kind,
                    start_time_unix_nano, end_time_unix_nano,
                    attributes, status, service_name,
                    events, links, trace_state,
                    dropped_attributes_count, dropped_events_count, dropped_links_count,
                    resource_attributes, scope
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
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_traces(
        &self,
        service_name: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Trace>> {
        let (query, params_vec) = service_filtered_query(
            "SELECT DISTINCT trace_id FROM spans",
            "start_time_unix_nano",
            service_name,
            limit,
        );

        let trace_ids: Vec<String> = {
            let conn = self.lock()?;
            let mut stmt = conn.prepare(&query)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(AsRef::as_ref).collect();
            stmt.query_map(&params_refs[..], |row| row.get(0))?
                .collect::<SqliteResult<Vec<_>>>()?
        };

        let mut traces = Vec::new();
        for trace_id in trace_ids {
            if let Ok(trace) = self.get_trace_by_id(&trace_id) {
                traces.push(trace);
            }
        }

        Ok(traces)
    }

    /// Get count of spans
    pub fn count_spans(&self) -> Result<i64> {
        self.count_rows("spans")
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
    fn test_span_full_fidelity_roundtrip() {
        use crate::models::{InstrumentationScope, SpanEvent, SpanLink};

        let storage = Storage::new_in_memory().unwrap();
        let mut event_attrs = Attributes::new();
        event_attrs.insert("exception.message", "boom");
        let mut resource_attrs = Attributes::new();
        resource_attrs.insert("service.version", "1.2.3");

        let span = create_test_span("span1", "trace1")
            .with_events(vec![SpanEvent {
                time_unix_nano: 1_000_000_000_000_000_050,
                name: "exception".to_string(),
                attributes: event_attrs,
                dropped_attributes_count: 2,
            }])
            .with_links(vec![SpanLink {
                trace_id: "other-trace".to_string(),
                span_id: "other-span".to_string(),
                trace_state: Some("vendor=1".to_string()),
                attributes: Attributes::new(),
                dropped_attributes_count: 0,
            }])
            .with_trace_state(Some("vendor=2".to_string()))
            .with_dropped_counts(1, 2, 3)
            .with_resource_attributes(resource_attrs)
            .with_scope(Some(InstrumentationScope::new(
                "test-lib".to_string(),
                Some("0.1".to_string()),
                Attributes::new(),
            )));

        storage.insert_span(&span).unwrap();
        let trace = storage.get_trace_by_id("trace1").unwrap();

        assert_eq!(trace.spans.len(), 1);
        assert_eq!(trace.spans[0], span);
    }

    #[test]
    fn test_get_nonexistent_trace() {
        let storage = Storage::new_in_memory().unwrap();
        let result = storage.get_trace_by_id("nonexistent");
        assert!(result.is_err());
    }
}
