use crate::convert::{
    ResourceContext, convert_any_value_to_string, convert_attributes, convert_resource_context,
    convert_scope, id_to_hex,
};
use crate::proto::opentelemetry::proto::logs::v1::{LogRecord, ResourceLogs};
use faze::models::InstrumentationScope as FazeScope;
use faze::models::log::{Log as FazeLog, SeverityLevel};

#[allow(clippy::cast_possible_wrap)]
fn convert_log(log: &LogRecord, resource: &ResourceContext, scope: Option<&FazeScope>) -> FazeLog {
    let trace_id = id_to_hex(&log.trace_id);
    let span_id = id_to_hex(&log.span_id);

    // OTLP severity numbers are the SeverityLevel discriminants (0..=24).
    let severity_level = SeverityLevel::from_severity_number(log.severity_number);
    // Keep the producer's severity text; fall back to the derived level name.
    let severity_text = if log.severity_text.is_empty() {
        Some(severity_level.as_str().to_string())
    } else {
        Some(log.severity_text.clone())
    };

    let attributes = convert_attributes(&log.attributes);

    let body = log
        .body
        .as_ref()
        .and_then(convert_any_value_to_string)
        .unwrap_or_default();

    let observed_time = if log.observed_time_unix_nano == 0 {
        None
    } else {
        Some(log.observed_time_unix_nano as i64)
    };
    let event_name = if log.event_name.is_empty() {
        None
    } else {
        Some(log.event_name.clone())
    };
    let flags = if log.flags == 0 {
        None
    } else {
        Some(log.flags)
    };

    FazeLog::new(
        log.time_unix_nano as i64,
        severity_level,
        severity_text,
        body,
        attributes,
        trace_id,
        span_id,
        resource.service_name.clone(),
    )
    .with_observed_time(observed_time)
    .with_event_name(event_name)
    .with_flags(flags)
    .with_resource_attributes(resource.attributes.clone())
    .with_scope(scope.cloned())
}

/// Convert OTLP `LogRecord` collections to internal `Log`s.
#[must_use]
pub fn convert_resource_logs(resource_logs: &[ResourceLogs]) -> Vec<FazeLog> {
    let mut logs = Vec::new();

    for rs in resource_logs {
        let resource = convert_resource_context(rs.resource.as_ref());

        for scope_logs in &rs.scope_logs {
            let scope = convert_scope(scope_logs.scope.as_ref());
            for record in &scope_logs.log_records {
                logs.push(convert_log(record, &resource, scope.as_ref()));
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
    fn test_convert_log_fidelity_fields() {
        let resource_logs = vec![ResourceLogs {
            resource: None,
            scope_logs: vec![ScopeLogs {
                scope: None,
                log_records: vec![LogRecord {
                    time_unix_nano: 1_000,
                    observed_time_unix_nano: 2_000,
                    severity_number: 13,
                    severity_text: "warning".to_string(),
                    event_name: "app.start".to_string(),
                    flags: 1,
                    trace_id: vec![0xab; 16],
                    span_id: vec![0xcd; 8],
                    ..Default::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }];

        let logs = convert_resource_logs(&resource_logs);
        let log = &logs[0];
        assert_eq!(log.observed_time_unix_nano, Some(2_000));
        // Producer severity text wins over the derived level name.
        assert_eq!(log.severity_text, Some("warning".to_string()));
        assert_eq!(log.event_name, Some("app.start".to_string()));
        assert_eq!(log.flags, Some(1));
        assert_eq!(log.trace_id, Some("ab".repeat(16)));
        assert_eq!(log.span_id, Some("cd".repeat(8)));
    }

    #[test]
    fn test_convert_log_empty_ids_and_defaults() {
        let resource_logs = vec![ResourceLogs {
            resource: None,
            scope_logs: vec![ScopeLogs {
                scope: None,
                log_records: vec![LogRecord {
                    time_unix_nano: 1_000,
                    severity_number: 9,
                    ..Default::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }];

        let logs = convert_resource_logs(&resource_logs);
        let log = &logs[0];
        // Empty ids map to None, not empty strings.
        assert_eq!(log.trace_id, None);
        assert_eq!(log.span_id, None);
        assert_eq!(log.observed_time_unix_nano, None);
        assert_eq!(log.event_name, None);
        assert_eq!(log.flags, None);
        // Missing severity text falls back to the derived level name.
        assert_eq!(log.severity_text, Some("INFO".to_string()));
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
