//! Aggregate statistics over traces: global totals, per-service counts,
//! and time-bucketed activity.

use super::{Result, Storage};
use rusqlite::{Result as SqliteResult, params};

/// Per-trace rollup shared by every stats query: one row per trace with its
/// start time, duration, error flag, and a representative service name.
const TRACE_ROLLUP: &str = "SELECT trace_id,
        MIN(start_time_unix_nano) AS st,
        MAX(end_time_unix_nano) - MIN(start_time_unix_nano) AS dur,
        MAX(CASE WHEN json_extract(status, '$.code') = 'ERROR' THEN 1 ELSE 0 END) AS has_err,
        MIN(service_name) AS service_name
     FROM spans
     GROUP BY trace_id";

/// Global trace aggregates.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceStats {
    /// Total number of distinct traces.
    pub total: i64,
    /// Number of traces containing at least one error span.
    pub errors: i64,
    /// Mean trace duration in milliseconds.
    pub avg_duration_ms: f64,
}

/// Per-service trace counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceStat {
    /// Service name.
    pub name: String,
    /// Number of traces attributed to the service.
    pub trace_count: i64,
    /// Number of those traces containing at least one error span.
    pub error_count: i64,
}

/// One time bucket of trace activity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceTimeBucket {
    /// Bucket start as nanoseconds since the Unix epoch.
    pub bucket_start_unix_nano: i64,
    /// Number of traces starting in this bucket.
    pub total: i64,
    /// Number of those traces containing at least one error span.
    pub errors: i64,
}

impl Storage {
    /// Global trace count, error count, and average duration.
    #[allow(clippy::significant_drop_tightening)]
    pub fn trace_stats(&self) -> Result<TraceStats> {
        let conn = self.lock()?;
        let stats = conn.query_row(
            &format!(
                "SELECT COUNT(*),
                        COALESCE(SUM(has_err), 0),
                        COALESCE(AVG(dur / 1000000.0), 0.0)
                 FROM ({TRACE_ROLLUP})"
            ),
            [],
            |row| {
                Ok(TraceStats {
                    total: row.get(0)?,
                    errors: row.get(1)?,
                    avg_duration_ms: row.get(2)?,
                })
            },
        )?;
        Ok(stats)
    }

    /// Trace and error counts per service, most active first.
    #[allow(clippy::significant_drop_tightening)]
    pub fn service_stats(&self) -> Result<Vec<ServiceStat>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT service_name, COUNT(*), COALESCE(SUM(has_err), 0)
             FROM ({TRACE_ROLLUP})
             WHERE service_name IS NOT NULL
             GROUP BY service_name
             ORDER BY COUNT(*) DESC, service_name"
        ))?;

        let stats = stmt
            .query_map([], |row| {
                Ok(ServiceStat {
                    name: row.get(0)?,
                    trace_count: row.get(1)?,
                    error_count: row.get(2)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(stats)
    }

    /// Trace activity grouped into `bucket_count` equal time buckets spanning
    /// the full range of trace start times. Buckets without traces are
    /// zero-filled; an empty database yields an empty `Vec`.
    #[allow(clippy::significant_drop_tightening)]
    pub fn trace_time_buckets(&self, bucket_count: usize) -> Result<Vec<TraceTimeBucket>> {
        if bucket_count == 0 {
            return Ok(Vec::new());
        }

        let conn = self.lock()?;
        let range: (Option<i64>, Option<i64>) = conn.query_row(
            &format!("SELECT MIN(st), MAX(st) FROM ({TRACE_ROLLUP})"),
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let (Some(min_start), Some(max_start)) = range else {
            return Ok(Vec::new());
        };

        let bucket_count = i64::try_from(bucket_count).unwrap_or(i64::MAX);
        // Width chosen so the latest trace still lands in the last bucket.
        let width = (max_start - min_start) / bucket_count + 1;

        let mut buckets: Vec<TraceTimeBucket> = (0..bucket_count)
            .map(|i| TraceTimeBucket {
                bucket_start_unix_nano: min_start + i * width,
                total: 0,
                errors: 0,
            })
            .collect();

        let mut stmt = conn.prepare(&format!(
            "SELECT (st - ?1) / ?2 AS bucket, COUNT(*), COALESCE(SUM(has_err), 0)
             FROM ({TRACE_ROLLUP})
             GROUP BY bucket"
        ))?;

        let rows = stmt.query_map(params![min_start, width], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        for row in rows {
            let (index, total, errors) = row?;
            if let Ok(index) = usize::try_from(index)
                && let Some(bucket) = buckets.get_mut(index)
            {
                bucket.total = total;
                bucket.errors = errors;
            }
        }

        Ok(buckets)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::models::{Attributes, Span, SpanKind, Status};

    fn create_span(
        span_id: &str,
        trace_id: &str,
        start: i64,
        end: i64,
        status: Status,
        service: &str,
    ) -> Span {
        Span::new(
            span_id.to_string(),
            trace_id.to_string(),
            None,
            "test-operation".to_string(),
            SpanKind::Server,
            start,
            end,
            Attributes::new(),
            status,
            Some(service.to_string()),
        )
    }

    #[test]
    fn test_trace_stats_empty() {
        let storage = Storage::new_in_memory().unwrap();
        let stats = storage.trace_stats().unwrap();

        assert_eq!(stats.total, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.avg_duration_ms, 0.0);
    }

    #[test]
    fn test_trace_stats_counts_and_avg() {
        let storage = Storage::new_in_memory().unwrap();
        // trace1: two spans, 100ms total, ok
        storage
            .insert_spans(&[
                create_span(
                    "s1",
                    "trace1",
                    1_000_000_000,
                    1_050_000_000,
                    Status::ok(),
                    "svc-a",
                ),
                create_span(
                    "s2",
                    "trace1",
                    1_020_000_000,
                    1_100_000_000,
                    Status::ok(),
                    "svc-a",
                ),
            ])
            .unwrap();
        // trace2: one span, 300ms, error
        storage
            .insert_spans(&[create_span(
                "s3",
                "trace2",
                2_000_000_000,
                2_300_000_000,
                Status::error("boom"),
                "svc-b",
            )])
            .unwrap();

        let stats = storage.trace_stats().unwrap();

        assert_eq!(stats.total, 2);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.avg_duration_ms, 200.0);
    }

    #[test]
    fn test_service_stats() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .insert_spans(&[
                create_span("s1", "trace1", 1_000, 2_000, Status::ok(), "svc-a"),
                create_span("s2", "trace2", 1_000, 2_000, Status::error("x"), "svc-a"),
                create_span("s3", "trace3", 1_000, 2_000, Status::ok(), "svc-b"),
            ])
            .unwrap();

        let stats = storage.service_stats().unwrap();

        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].name, "svc-a");
        assert_eq!(stats[0].trace_count, 2);
        assert_eq!(stats[0].error_count, 1);
        assert_eq!(stats[1].name, "svc-b");
        assert_eq!(stats[1].trace_count, 1);
        assert_eq!(stats[1].error_count, 0);
    }

    #[test]
    fn test_trace_time_buckets_empty() {
        let storage = Storage::new_in_memory().unwrap();
        let buckets = storage.trace_time_buckets(30).unwrap();
        assert!(buckets.is_empty());
    }

    #[test]
    fn test_trace_time_buckets_distribution() {
        let storage = Storage::new_in_memory().unwrap();
        // Two traces at the start of the range, one error at the end.
        storage
            .insert_spans(&[
                create_span("s1", "trace1", 0, 1_000, Status::ok(), "svc"),
                create_span("s2", "trace2", 100, 1_000, Status::ok(), "svc"),
                create_span("s3", "trace3", 10_000, 11_000, Status::error("x"), "svc"),
            ])
            .unwrap();

        let buckets = storage.trace_time_buckets(10).unwrap();

        assert_eq!(buckets.len(), 10);
        assert_eq!(buckets.iter().map(|b| b.total).sum::<i64>(), 3);
        assert_eq!(buckets.iter().map(|b| b.errors).sum::<i64>(), 1);
        assert_eq!(buckets[0].bucket_start_unix_nano, 0);
        // First bucket holds the two early traces, last non-empty one the error.
        assert_eq!(buckets[0].total, 2);
        assert_eq!(buckets.last().unwrap().errors + buckets[8].errors, 1);
    }

    #[test]
    fn test_trace_time_buckets_single_trace() {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .insert_spans(&[create_span(
                "s1",
                "trace1",
                5_000,
                6_000,
                Status::ok(),
                "svc",
            )])
            .unwrap();

        let buckets = storage.trace_time_buckets(5).unwrap();

        assert_eq!(buckets.len(), 5);
        assert_eq!(buckets[0].bucket_start_unix_nano, 5_000);
        assert_eq!(buckets[0].total, 1);
        assert_eq!(buckets.iter().map(|b| b.total).sum::<i64>(), 1);
    }
}
