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

#[allow(clippy::match_same_arms)]
pub fn parse_metric_type(type_str: &str) -> MetricType {
    match type_str {
        "Gauge" => MetricType::Gauge,
        "Sum" => MetricType::Sum,
        "Histogram" => MetricType::Histogram,
        "Summary" => MetricType::Summary,
        _ => MetricType::Gauge,
    }
}

#[allow(clippy::match_same_arms)]
pub fn parse_temporality(temporality_str: &str) -> AggregationTemporality {
    match temporality_str {
        "Unspecified" => AggregationTemporality::Unspecified,
        "Delta" => AggregationTemporality::Delta,
        "Cumulative" => AggregationTemporality::Cumulative,
        _ => AggregationTemporality::Unspecified,
    }
}

#[allow(clippy::match_same_arms)]
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

#[allow(clippy::match_same_arms)]
pub fn parse_severity_level(severity_str: &str) -> SeverityLevel {
    match severity_str {
        "Unspecified" => SeverityLevel::Unspecified,
        "Trace" => SeverityLevel::Trace,
        "Trace2" => SeverityLevel::Trace2,
        "Trace3" => SeverityLevel::Trace3,
        "Trace4" => SeverityLevel::Trace4,
        "Debug" => SeverityLevel::Debug,
        "Debug2" => SeverityLevel::Debug2,
        "Debug3" => SeverityLevel::Debug3,
        "Debug4" => SeverityLevel::Debug4,
        "Info" => SeverityLevel::Info,
        "Info2" => SeverityLevel::Info2,
        "Info3" => SeverityLevel::Info3,
        "Info4" => SeverityLevel::Info4,
        "Warn" => SeverityLevel::Warn,
        "Warn2" => SeverityLevel::Warn2,
        "Warn3" => SeverityLevel::Warn3,
        "Warn4" => SeverityLevel::Warn4,
        "Error" => SeverityLevel::Error,
        "Error2" => SeverityLevel::Error2,
        "Error3" => SeverityLevel::Error3,
        "Error4" => SeverityLevel::Error4,
        "Fatal" => SeverityLevel::Fatal,
        "Fatal2" => SeverityLevel::Fatal2,
        "Fatal3" => SeverityLevel::Fatal3,
        "Fatal4" => SeverityLevel::Fatal4,
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
