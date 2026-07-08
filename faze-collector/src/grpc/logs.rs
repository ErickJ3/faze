use crate::{
    convert::logs::convert_resource_logs,
    proto::opentelemetry::proto::collector::logs::v1::{
        ExportLogsPartialSuccess, ExportLogsServiceRequest, ExportLogsServiceResponse,
        logs_service_server::{LogsService, LogsServiceServer},
    },
};
use faze::Storage;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::error;

/// OTLP collector that receives logs via gRPC
pub struct OtlpLogsCollector {
    storage: Arc<Storage>,
}

impl OtlpLogsCollector {
    /// Build a collector backed by the given storage handle.
    #[must_use]
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    /// Wrap the collector into a tonic `LogsServiceServer`.
    #[must_use]
    pub fn into_service(self) -> LogsServiceServer<Self> {
        LogsServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl LogsService for OtlpLogsCollector {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let req = request.into_inner();
        let logs = convert_resource_logs(&req.resource_logs);

        // Logs are inserted as one transaction; a failure rejects the whole batch.
        let partial_success = match self.storage.insert_logs(&logs) {
            Ok(()) => None,
            Err(e) => {
                error!("Failed to insert {} logs: {e}", logs.len());
                Some(ExportLogsPartialSuccess {
                    rejected_log_records: i64::try_from(logs.len()).unwrap_or(i64::MAX),
                    error_message: e.to_string(),
                })
            }
        };

        Ok(Response::new(ExportLogsServiceResponse { partial_success }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::proto::{
        common::v1::{AnyValue, KeyValue, any_value},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    };
    use faze::SeverityLevel;

    fn create_test_log_record(body: &str, severity_number: i32) -> LogRecord {
        LogRecord {
            time_unix_nano: 1_000_000_000_000_000_000,
            severity_number,
            body: Some(AnyValue {
                value: Some(any_value::Value::StringValue(body.to_string())),
            }),
            ..Default::default()
        }
    }

    fn create_test_request(records: Vec<LogRecord>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: records,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    #[tokio::test]
    async fn test_export_logs() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpLogsCollector::new(storage.clone());

        let request = create_test_request(vec![create_test_log_record("hello", 9)]);
        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_logs().unwrap(), 1);

        let logs = storage.list_logs(None, None).unwrap();
        assert_eq!(logs[0].body, "hello");
        assert_eq!(logs[0].severity_level, SeverityLevel::Info);
        assert_eq!(logs[0].service_name, Some("test-service".to_string()));
    }

    #[tokio::test]
    async fn test_export_multiple_logs() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpLogsCollector::new(storage.clone());

        let request = create_test_request(vec![
            create_test_log_record("first", 9),
            create_test_log_record("second", 17),
        ]);
        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_logs().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_export_empty_request() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpLogsCollector::new(storage.clone());

        let request = ExportLogsServiceRequest {
            resource_logs: vec![],
        };
        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_logs().unwrap(), 0);
    }
}
