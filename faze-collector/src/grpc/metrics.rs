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
    /// Build a collector backed by the given storage handle.
    #[must_use]
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    /// Wrap the collector into a tonic `MetricsServiceServer`.
    #[must_use]
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

        // Metrics are inserted as one transaction; a failure rejects the whole batch.
        let partial_success = match self.storage.insert_metrics(&metrics) {
            Ok(()) => None,
            Err(e) => {
                error!("Failed to insert {} metrics: {e}", metrics.len());
                let data_points: usize = metrics.iter().map(|m| m.data_points.len()).sum();
                Some(ExportMetricsPartialSuccess {
                    rejected_data_points: i64::try_from(data_points).unwrap_or(i64::MAX),
                    error_message: e.to_string(),
                })
            }
        };

        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::proto::{
        common::v1::{AnyValue, KeyValue, any_value},
        metrics::v1::{
            Gauge, Metric as OtlpMetric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric,
            number_data_point,
        },
        resource::v1::Resource,
    };
    use faze::MetricType;

    fn create_test_metric(name: &str, value: f64) -> OtlpMetric {
        OtlpMetric {
            name: name.to_string(),
            description: "test metric".to_string(),
            unit: "1".to_string(),
            data: Some(metric::Data::Gauge(Gauge {
                data_points: vec![NumberDataPoint {
                    time_unix_nano: 1_000_000_000_000_000_000,
                    value: Some(number_data_point::Value::AsDouble(value)),
                    ..Default::default()
                }],
            })),
            ..Default::default()
        }
    }

    fn create_test_request(metrics: Vec<OtlpMetric>) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    #[tokio::test]
    async fn test_export_metrics() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpMetricsCollector::new(storage.clone());

        let request = create_test_request(vec![create_test_metric("cpu.usage", 0.75)]);
        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_metrics().unwrap(), 1);

        let metrics = storage.list_metrics(None, None).unwrap();
        assert_eq!(metrics[0].name, "cpu.usage");
        assert_eq!(metrics[0].metric_type, MetricType::Gauge);
        assert_eq!(metrics[0].service_name, Some("test-service".to_string()));
    }

    #[tokio::test]
    async fn test_export_multiple_metrics() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpMetricsCollector::new(storage.clone());

        let request = create_test_request(vec![
            create_test_metric("cpu.usage", 0.5),
            create_test_metric("mem.usage", 0.25),
        ]);
        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_metrics().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_export_empty_request() {
        let storage = Storage::new_in_memory().unwrap();
        let collector = OtlpMetricsCollector::new(storage.clone());

        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![],
        };
        let response = collector
            .export(Request::new(request))
            .await
            .unwrap()
            .into_inner();

        assert!(response.partial_success.is_none());
        assert_eq!(storage.count_metrics().unwrap(), 0);
    }
}
