use crate::models::{
    AggregationTemporality, Attributes, Distribution, Exemplar, InstrumentationScope, Log, Metric,
    MetricDataPoint, MetricType, SeverityLevel, Span, SpanEvent, SpanKind, SpanLink, Status,
};
use rusqlite::Row;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn from_json<'de, T: Deserialize<'de>>(json: &'de str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize a slice to JSON, storing NULL when empty.
pub fn slice_to_json<T: Serialize>(items: &[T]) -> Result<Option<String>, serde_json::Error> {
    if items.is_empty() {
        Ok(None)
    } else {
        to_json(&items).map(Some)
    }
}

/// Serialize attributes to JSON, storing NULL when empty.
pub fn attrs_to_json(attrs: &Attributes) -> Result<Option<String>, serde_json::Error> {
    if attrs.is_empty() {
        Ok(None)
    } else {
        to_json(attrs).map(Some)
    }
}

/// Serialize an optional value to JSON, storing NULL when absent.
pub fn opt_to_json<T: Serialize>(value: Option<&T>) -> Result<Option<String>, serde_json::Error> {
    value.map(to_json).transpose()
}

fn conversion_err(idx: usize, e: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
}

fn json_col(row: &Row, idx: usize) -> rusqlite::Result<Attributes> {
    let json: String = row.get(idx)?;
    from_json(&json).map_err(|e| conversion_err(idx, e))
}

/// Read a nullable JSON column, defaulting when NULL.
fn opt_json_col<T: DeserializeOwned + Default>(row: &Row, idx: usize) -> rusqlite::Result<T> {
    let json: Option<String> = row.get(idx)?;
    json.map_or_else(
        || Ok(T::default()),
        |j| from_json(&j).map_err(|e| conversion_err(idx, e)),
    )
}

pub fn span_from_row(row: &Row) -> rusqlite::Result<Span> {
    let attributes = json_col(row, 7)?;
    let status_json: String = row.get(8)?;
    let status: Status = from_json(&status_json).map_err(|e| conversion_err(8, e))?;

    let kind_str: String = row.get(4)?;
    let kind = SpanKind::from_db_str(&kind_str);

    let events: Vec<SpanEvent> = opt_json_col(row, 10)?;
    let links: Vec<SpanLink> = opt_json_col(row, 11)?;
    let resource_attributes: Attributes = opt_json_col(row, 16)?;
    let scope: Option<InstrumentationScope> = opt_json_col(row, 17)?;

    Ok(Span::new(
        row.get(0)?, // span_id
        row.get(1)?, // trace_id
        row.get(2)?, // parent_span_id
        row.get(3)?, // name
        kind,
        row.get(5)?, // start_time_unix_nano
        row.get(6)?, // end_time_unix_nano
        attributes,
        status,
        row.get(9)?, // service_name
    )
    .with_events(events)
    .with_links(links)
    .with_trace_state(row.get(12)?)
    .with_dropped_counts(row.get(13)?, row.get(14)?, row.get(15)?)
    .with_resource_attributes(resource_attributes)
    .with_scope(scope))
}

pub fn log_from_row(row: &Row) -> rusqlite::Result<Log> {
    let attributes = json_col(row, 4)?;
    let severity_str: String = row.get(1)?;

    let resource_attributes: Attributes = opt_json_col(row, 11)?;
    let scope: Option<InstrumentationScope> = opt_json_col(row, 12)?;

    Ok(Log::new(
        row.get(0)?, // time_unix_nano
        SeverityLevel::from_db_str(&severity_str),
        row.get(2)?, // severity_text
        row.get(3)?, // body
        attributes,
        row.get(5)?, // trace_id
        row.get(6)?, // span_id
        row.get(7)?, // service_name
    )
    .with_observed_time(row.get(8)?)
    .with_event_name(row.get(9)?)
    .with_flags(row.get(10)?)
    .with_resource_attributes(resource_attributes)
    .with_scope(scope))
}

/// Build a single-data-point `Metric` from a metrics table row.
pub fn metric_from_row(row: &Row) -> rusqlite::Result<Metric> {
    let attributes = json_col(row, 8)?;

    let metric_type_str: String = row.get(3)?;
    let temporality_str: String = row.get(4)?;

    let distribution: Option<Distribution> = opt_json_col(row, 11)?;
    let exemplars: Vec<Exemplar> = opt_json_col(row, 12)?;
    let resource_attributes: Attributes = opt_json_col(row, 13)?;
    let scope: Option<InstrumentationScope> = opt_json_col(row, 14)?;

    let data_point = MetricDataPoint::new(
        row.get(5)?, // time_unix_nano
        row.get(6)?, // start_time_unix_nano
        row.get(7)?, // value
        attributes,
    )
    .with_distribution(distribution)
    .with_exemplars(exemplars);

    Ok(Metric::new(
        row.get(0)?, // name
        row.get(1)?, // description
        row.get(2)?, // unit
        MetricType::from_db_str(&metric_type_str),
        AggregationTemporality::from_db_str(&temporality_str),
        vec![data_point],
        row.get(9)?, // service_name
    )
    .with_is_monotonic(row.get(10)?)
    .with_resource_attributes(resource_attributes)
    .with_scope(scope))
}
