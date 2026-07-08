use super::attributes::Attributes;
use super::db_enum::impl_db_str;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricType {
    /// Single point in time numeric value.
    Gauge,
    /// Monotonic or non-monotonic cumulative/delta sum.
    Sum,
    /// Bucketed distribution of values.
    Histogram,
    /// Summary statistics (quantiles).
    Summary,
}

impl_db_str!(
    MetricType {
        Gauge,
        Sum,
        Histogram,
        Summary,
    },
    fallback = Gauge
);

/// Aggregation temporality for Sum and Histogram metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AggregationTemporality {
    /// Temporality is not specified.
    #[default]
    Unspecified,
    /// Data point reports the change since the last reset.
    Delta,
    /// Data point reports the value accumulated since process start.
    Cumulative,
}

impl_db_str!(
    AggregationTemporality {
        Unspecified,
        Delta,
        Cumulative,
    },
    fallback = Unspecified
);

/// Represents a metric data point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// Timestamp (nanoseconds since epoch)
    pub time_unix_nano: i64,
    /// Start time for cumulative metrics (nanoseconds since epoch)
    pub start_time_unix_nano: Option<i64>,
    /// Numeric value
    pub value: f64,
    /// Data point attributes
    pub attributes: Attributes,
}

impl MetricDataPoint {
    /// Build a single data point.
    #[must_use]
    pub const fn new(
        time_unix_nano: i64,
        start_time_unix_nano: Option<i64>,
        value: f64,
        attributes: Attributes,
    ) -> Self {
        Self {
            time_unix_nano,
            start_time_unix_nano,
            value,
            attributes,
        }
    }

    /// Get timestamp as `DateTime`
    #[must_use]
    pub const fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_nanos(self.time_unix_nano)
    }

    /// Get start time as `DateTime` (if available)
    #[must_use]
    pub fn start_time(&self) -> Option<DateTime<Utc>> {
        self.start_time_unix_nano
            .map(DateTime::from_timestamp_nanos)
    }
}

/// Represents a metric
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name (e.g., "http.server.duration", "system.cpu.utilization")
    pub name: String,
    /// Description of the metric
    pub description: Option<String>,
    /// Unit of measurement (e.g., "ms", "bytes", "1")
    pub unit: Option<String>,
    /// Type of metric
    pub metric_type: MetricType,
    /// Aggregation temporality (for Sum and Histogram)
    pub temporality: AggregationTemporality,
    /// Data points
    pub data_points: Vec<MetricDataPoint>,
    /// Service name (denormalized from resource)
    pub service_name: Option<String>,
}

impl Metric {
    /// Build a metric from its component fields.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        name: String,
        description: Option<String>,
        unit: Option<String>,
        metric_type: MetricType,
        temporality: AggregationTemporality,
        data_points: Vec<MetricDataPoint>,
        service_name: Option<String>,
    ) -> Self {
        Self {
            name,
            description,
            unit,
            metric_type,
            temporality,
            data_points,
            service_name,
        }
    }

    /// Create a gauge metric
    #[must_use]
    pub const fn gauge(
        name: String,
        data_points: Vec<MetricDataPoint>,
        service_name: Option<String>,
    ) -> Self {
        Self::new(
            name,
            None,
            None,
            MetricType::Gauge,
            AggregationTemporality::Unspecified,
            data_points,
            service_name,
        )
    }

    /// Create a counter (cumulative sum) metric
    #[must_use]
    pub const fn counter(
        name: String,
        data_points: Vec<MetricDataPoint>,
        service_name: Option<String>,
    ) -> Self {
        Self::new(
            name,
            None,
            None,
            MetricType::Sum,
            AggregationTemporality::Cumulative,
            data_points,
            service_name,
        )
    }

    /// Get the latest value (if any data points exist)
    #[must_use]
    pub fn latest_value(&self) -> Option<f64> {
        self.data_points.last().map(|dp| dp.value)
    }

    /// Get the number of data points
    #[must_use]
    pub const fn data_point_count(&self) -> usize {
        self.data_points.len()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn create_test_data_point(value: f64) -> MetricDataPoint {
        MetricDataPoint::new(
            1_000_000_000_000_000_000,
            Some(1_000_000_000_000_000_000 - 1_000_000_000),
            value,
            Attributes::new(),
        )
    }

    #[test]
    fn test_metric_data_point_creation() {
        let dp = create_test_data_point(42.5);
        assert_eq!(dp.value, 42.5);
        assert!(dp.start_time_unix_nano.is_some());
    }

    #[test]
    fn test_metric_creation() {
        let data_points = vec![create_test_data_point(10.0), create_test_data_point(20.0)];

        let metric = Metric::new(
            "http.request.duration".to_string(),
            Some("HTTP request duration".to_string()),
            Some("ms".to_string()),
            MetricType::Histogram,
            AggregationTemporality::Delta,
            data_points,
            Some("api-service".to_string()),
        );

        assert_eq!(metric.name, "http.request.duration");
        assert_eq!(
            metric.description,
            Some("HTTP request duration".to_string())
        );
        assert_eq!(metric.unit, Some("ms".to_string()));
        assert_eq!(metric.metric_type, MetricType::Histogram);
        assert_eq!(metric.temporality, AggregationTemporality::Delta);
        assert_eq!(metric.data_points.len(), 2);
    }

    #[test]
    fn test_metric_gauge() {
        let data_points = vec![create_test_data_point(75.5)];
        let metric = Metric::gauge(
            "system.cpu.utilization".to_string(),
            data_points,
            Some("host-1".to_string()),
        );

        assert_eq!(metric.metric_type, MetricType::Gauge);
        assert_eq!(metric.name, "system.cpu.utilization");
    }

    #[test]
    fn test_metric_counter() {
        let data_points = vec![create_test_data_point(100.0)];
        let metric = Metric::counter(
            "http.requests.total".to_string(),
            data_points,
            Some("api-service".to_string()),
        );

        assert_eq!(metric.metric_type, MetricType::Sum);
        assert_eq!(metric.temporality, AggregationTemporality::Cumulative);
        assert_eq!(metric.name, "http.requests.total");
    }

    #[test]
    fn test_metric_latest_value() {
        let data_points = vec![
            create_test_data_point(10.0),
            create_test_data_point(20.0),
            create_test_data_point(30.0),
        ];

        let metric = Metric::gauge("test".to_string(), data_points, None);
        assert_eq!(metric.latest_value(), Some(30.0));
    }

    #[test]
    fn test_metric_latest_value_empty() {
        let metric = Metric::gauge("test".to_string(), vec![], None);
        assert_eq!(metric.latest_value(), None);
    }

    #[test]
    fn test_metric_data_point_count() {
        let data_points = vec![create_test_data_point(1.0), create_test_data_point(2.0)];
        let metric = Metric::gauge("test".to_string(), data_points, None);
        assert_eq!(metric.data_point_count(), 2);
    }

    #[test]
    fn test_metric_serde() {
        let data_points = vec![create_test_data_point(42.0)];
        let metric = Metric::gauge("test".to_string(), data_points, Some("svc".to_string()));

        let json = serde_json::to_string(&metric).unwrap();
        let deserialized: Metric = serde_json::from_str(&json).unwrap();
        assert_eq!(metric, deserialized);
    }

    #[test]
    fn test_metric_with_attributes() {
        let mut attrs = Attributes::new();
        attrs.insert("method", "GET");
        attrs.insert("status", 200i64);

        let dp = MetricDataPoint::new(1_000_000_000, None, 150.5, attrs.clone());
        let metric = Metric::gauge("http.duration".to_string(), vec![dp], None);

        assert_eq!(
            metric.data_points[0].attributes.get_string("method"),
            Some("GET")
        );
        assert_eq!(
            metric.data_points[0].attributes.get_int("status"),
            Some(200)
        );
    }
}
