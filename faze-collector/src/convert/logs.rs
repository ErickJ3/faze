use crate::convert::{
    bytes_to_hex, convert_any_value_to_string, convert_attributes, convert_resource,
};
use crate::proto::opentelemetry::proto::logs::v1::{
    LogRecord, ResourceLogs, SeverityNumber as OtlSeveryNumber,
};
use faze::models::log::{Log as FazeLog, SeverityLevel};

// Convert OTLP SeverityNumber to internal SeverityLevel
pub fn convert_log_severity_level(kind: i32) -> SeverityLevel {
    match OtlSeveryNumber::try_from(kind) {
        Ok(OtlSeveryNumber::Unspecified) => SeverityLevel::Unspecified,
        Ok(OtlSeveryNumber::Trace) => SeverityLevel::Trace,
        Ok(OtlSeveryNumber::Trace2) => SeverityLevel::Trace2,
        Ok(OtlSeveryNumber::Trace3) => SeverityLevel::Trace3,
        Ok(OtlSeveryNumber::Trace4) => SeverityLevel::Trace4,
        Ok(OtlSeveryNumber::Debug) => SeverityLevel::Debug,
        Ok(OtlSeveryNumber::Debug2) => SeverityLevel::Debug2,
        Ok(OtlSeveryNumber::Debug3) => SeverityLevel::Debug3,
        Ok(OtlSeveryNumber::Debug4) => SeverityLevel::Debug4,
        Ok(OtlSeveryNumber::Info) => SeverityLevel::Info,
        Ok(OtlSeveryNumber::Info2) => SeverityLevel::Info2,
        Ok(OtlSeveryNumber::Info3) => SeverityLevel::Info3,
        Ok(OtlSeveryNumber::Info4) => SeverityLevel::Info4,
        Ok(OtlSeveryNumber::Warn) => SeverityLevel::Warn,
        Ok(OtlSeveryNumber::Warn2) => SeverityLevel::Warn2,
        Ok(OtlSeveryNumber::Warn3) => SeverityLevel::Warn3,
        Ok(OtlSeveryNumber::Warn4) => SeverityLevel::Warn4,
        Ok(OtlSeveryNumber::Error) => SeverityLevel::Error,
        Ok(OtlSeveryNumber::Error2) => SeverityLevel::Error2,
        Ok(OtlSeveryNumber::Error3) => SeverityLevel::Error3,
        Ok(OtlSeveryNumber::Error4) => SeverityLevel::Error4,
        Ok(OtlSeveryNumber::Fatal) => SeverityLevel::Fatal,
        Ok(OtlSeveryNumber::Fatal2) => SeverityLevel::Fatal2,
        Ok(OtlSeveryNumber::Fatal3) => SeverityLevel::Fatal3,
        Ok(OtlSeveryNumber::Fatal4) => SeverityLevel::Fatal4,
        Err(_) => SeverityLevel::Unspecified,
    }
}

// Convert OTLP LogRecord to internal Log
fn convert_log(log: &LogRecord, service_name: Option<String>) -> FazeLog {
    let trace_id = Some(bytes_to_hex(&log.trace_id));
    let span_id = Some(bytes_to_hex(&log.span_id));

    let severity_level = convert_log_severity_level(log.severity_number);
    let severity_text = Some(severity_level.as_str().to_string());

    let attributes = convert_attributes(&log.attributes);

    let body = log
        .body
        .as_ref()
        .and_then(convert_any_value_to_string)
        .unwrap_or_default();

    FazeLog::new(
        log.time_unix_nano as i64,
        severity_level,
        severity_text,
        body,
        attributes,
        trace_id,
        span_id,
        service_name,
    )
}

/// Convert OTLP LogRecord to internal Log
pub fn convert_resource_logs(resource_logs: &[ResourceLogs]) -> Vec<FazeLog> {
    let mut logs = Vec::new();

    for rs in resource_logs {
        let service_name = rs
            .resource
            .as_ref()
            .map(convert_resource)
            .and_then(|r| r.service_name().map(|s| s.to_string()));

        for scope_logs in &rs.scope_logs {
            for span in &scope_logs.log_records {
                logs.push(convert_log(span, service_name.clone()));
            }
        }
    }

    logs
}
