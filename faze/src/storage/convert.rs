use crate::models::{
    AggregationTemporality, Attributes, MetricType, SeverityLevel, Span, SpanKind, Status,
};
use rusqlite::Row;
use serde::{Deserialize, Serialize};

pub fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn from_json<'de, T: Deserialize<'de>>(json: &'de str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn parse_metric_type(type_str: &str) -> MetricType {
    match type_str {
        "Gauge" => MetricType::Gauge,
        "Sum" => MetricType::Sum,
        "Histogram" => MetricType::Histogram,
        "Summary" => MetricType::Summary,
        _ => MetricType::Gauge,
    }
}

pub fn parse_temporality(temporality_str: &str) -> AggregationTemporality {
    match temporality_str {
        "Unspecified" => AggregationTemporality::Unspecified,
        "Delta" => AggregationTemporality::Delta,
        "Cumulative" => AggregationTemporality::Cumulative,
        _ => AggregationTemporality::Unspecified,
    }
}
pub fn parse_span_kind(kind_str: &str) -> SpanKind {
    match kind_str {
        "Unspecified" => SpanKind::Unspecified,
        "Internal" => SpanKind::Internal,
        "Server" => SpanKind::Server,
        "Client" => SpanKind::Client,
        "Producer" => SpanKind::Producer,
        "Consumer" => SpanKind::Consumer,
        _ => SpanKind::Unspecified,
    }
}

pub fn parse_severity_level(severity_str: &str) -> SeverityLevel {
    match severity_str {
        "Info" => SeverityLevel::Info,
        "Warn" => SeverityLevel::Warn,
        "Error" => SeverityLevel::Error,
        "Debug" => SeverityLevel::Debug,
        "Trace" => SeverityLevel::Trace,
        "Fatal" => SeverityLevel::Fatal,
        _ => SeverityLevel::Unspecified,
    }
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
    let kind = parse_span_kind(&kind_str);

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
