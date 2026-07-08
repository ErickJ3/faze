use crate::convert::{
    bytes_to_hex, convert_any_value_to_string, convert_attributes, resource_service_name,
};
use crate::proto::opentelemetry::proto::logs::v1::{LogRecord, ResourceLogs};
use faze::models::log::{Log as FazeLog, SeverityLevel};

#[allow(clippy::cast_possible_wrap)]
fn convert_log(log: &LogRecord, service_name: Option<String>) -> FazeLog {
    let trace_id = Some(bytes_to_hex(&log.trace_id));
    let span_id = Some(bytes_to_hex(&log.span_id));

    // OTLP severity numbers are the SeverityLevel discriminants (0..=24).
    let severity_level = SeverityLevel::from_severity_number(log.severity_number);
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

/// Convert OTLP `LogRecord` collections to internal `Log`s.
#[must_use]
pub fn convert_resource_logs(resource_logs: &[ResourceLogs]) -> Vec<FazeLog> {
    let mut logs = Vec::new();

    for rs in resource_logs {
        let service_name = resource_service_name(rs.resource.as_ref());

        for scope_logs in &rs.scope_logs {
            for record in &scope_logs.log_records {
                logs.push(convert_log(record, service_name.clone()));
            }
        }
    }

    logs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::proto::{
        common::v1::{AnyValue, KeyValue, any_value},
        logs::v1::ScopeLogs,
        resource::v1::Resource,
    };

    #[test]
    fn test_severity_number_mapping() {
        // OTLP severity numbers map 1:1 onto SeverityLevel discriminants.
        assert_eq!(
            SeverityLevel::from_severity_number(0),
            SeverityLevel::Unspecified
        );
        assert_eq!(SeverityLevel::from_severity_number(1), SeverityLevel::Trace);
        assert_eq!(SeverityLevel::from_severity_number(9), SeverityLevel::Info);
        assert_eq!(
            SeverityLevel::from_severity_number(17),
            SeverityLevel::Error
        );
        assert_eq!(
            SeverityLevel::from_severity_number(24),
            SeverityLevel::Fatal4
        );
        assert_eq!(
            SeverityLevel::from_severity_number(25),
            SeverityLevel::Unspecified
        );
        assert_eq!(
            SeverityLevel::from_severity_number(-1),
            SeverityLevel::Unspecified
        );
    }

    #[test]
    fn test_convert_resource_logs() {
        let resource_logs = vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("svc".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
            }),
            scope_logs: vec![ScopeLogs {
                scope: None,
                log_records: vec![LogRecord {
                    time_unix_nano: 1_000_000_000,
                    severity_number: 13,
                    body: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("warn message".to_string())),
                    }),
                    ..Default::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }];

        let logs = convert_resource_logs(&resource_logs);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].body, "warn message");
        assert_eq!(logs[0].severity_level, SeverityLevel::Warn);
        assert_eq!(logs[0].service_name, Some("svc".to_string()));
    }
}
