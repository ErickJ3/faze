use crate::{
    convert::metrics::convert_resource_metrics,
    proto::opentelemetry::proto::collector::metrics::v1::{
        ExportMetricsPartialSuccess, ExportMetricsServiceRequest, ExportMetricsServiceResponse,
        metrics_service_server::{MetricsService, MetricsServiceServer},
    },
};
use faze::Storage;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::error;

/// OTLP collector that receives metrics via gRPC
pub struct OtlpMetricsCollector {
    storage: Arc<Storage>,
}

impl OtlpMetricsCollector {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    pub fn into_service(self) -> MetricsServiceServer<Self> {
        MetricsServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl MetricsService for OtlpMetricsCollector {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let req = request.into_inner();

        let metrics = convert_resource_metrics(req.resource_metrics);

        let mut rejected_data_points = 0;
        let mut error_messages = Vec::new();

        for metric in metrics {
            let points_count = metric.data_points.len() as i64;

            if let Err(e) = self.storage.insert_metric(&metric) {
                error!("Failed to insert metric: {}", e);

                rejected_data_points += points_count;

                if error_messages.len() < 5 {
                    error_messages.push(format!("Error inserting metric: {}", e));
                }
            }
        }

        let response = if rejected_data_points > 0 {
            ExportMetricsServiceResponse {
                partial_success: Some(ExportMetricsPartialSuccess {
                    rejected_data_points,
                    error_message: error_messages.join("; "),
                }),
            }
        } else {
            ExportMetricsServiceResponse {
                partial_success: None,
            }
        };

        Ok(Response::new(response))
    }
}
