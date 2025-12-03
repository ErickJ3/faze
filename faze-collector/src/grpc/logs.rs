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
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

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

        let mut rejected_log_records = 0;
        let mut error_messages = Vec::new();

        for log in &logs {
            if let Err(e) = self.storage.insert_log(log) {
                error!("Failed to insert span {:?}: {}", log.span_id, e);
                rejected_log_records += 1;
                error_messages.push(format!("span {:?}: {}", log.span_id, e));
            }
        }

        let response = if rejected_log_records > 0 {
            ExportLogsServiceResponse {
                partial_success: Some(ExportLogsPartialSuccess {
                    rejected_log_records,
                    error_message: error_messages.join("; "),
                }),
            }
        } else {
            ExportLogsServiceResponse {
                partial_success: None,
            }
        };

        Ok(Response::new(response))
    }
}
