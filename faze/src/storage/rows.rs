use crate::models::{
    AggregationTemporality, Attributes, Log, Metric, MetricDataPoint, MetricType, SeverityLevel,
    Span, SpanKind, Status,
};
use rusqlite::Row;
use serde::{Deserialize, Serialize};

pub fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn from_json<'de, T: Deserialize<'de>>(json: &'de str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn span_from_row(row: &Row) -> rusqlite::Result<Span> {
    let attributes_json: String = row.get(7)?;
    let status_json: String = row.get(8)?;

    let attributes: Attributes = from_json(&attributes_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let status: Status = from_json(&status_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let kind_str: String = row.get(4)?;
    let kind = SpanKind::from_db_str(&kind_str);

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
    ))
}

pub fn log_from_row(row: &Row) -> rusqlite::Result<Log> {
    let attributes_json: String = row.get(4)?;
    let attributes: Attributes = from_json(&attributes_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let severity_str: String = row.get(1)?;

    Ok(Log::new(
        row.get(0)?, // time_unix_nano
        SeverityLevel::from_db_str(&severity_str),
        row.get(2)?, // severity_text
        row.get(3)?, // body
        attributes,
        row.get(5)?, // trace_id
        row.get(6)?, // span_id
        row.get(7)?, // service_name
    ))
}

/// Build a single-data-point `Metric` from a metrics table row.
pub fn metric_from_row(row: &Row) -> rusqlite::Result<Metric> {
    let attributes_json: String = row.get(8)?;
    let attributes: Attributes = from_json(&attributes_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let metric_type_str: String = row.get(3)?;
    let temporality_str: String = row.get(4)?;

    let data_point = MetricDataPoint::new(
        row.get(5)?, // time_unix_nano
        row.get(6)?, // start_time_unix_nano
        row.get(7)?, // value
        attributes,
    );

    Ok(Metric::new(
        row.get(0)?, // name
        row.get(1)?, // description
        row.get(2)?, // unit
        MetricType::from_db_str(&metric_type_str),
        AggregationTemporality::from_db_str(&temporality_str),
        vec![data_point],
        row.get(9)?, // service_name
    ))
}
