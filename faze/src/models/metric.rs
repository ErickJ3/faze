use super::attributes::Attributes;
use super::db_enum::impl_db_str;
use super::scope::InstrumentationScope;
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
    /// Base-2 exponentially scaled bucketed distribution.
    ExponentialHistogram,
}

impl_db_str!(
    MetricType {
        Gauge,
        Sum,
        Histogram,
        Summary,
        ExponentialHistogram,
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

/// Quantile of a summary metric
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantileValue {
    /// Quantile in [0.0, 1.0] (e.g., 0.99)
    pub quantile: f64,
    /// Value at the quantile
    pub value: f64,
}

/// Example measurement linked to a data point, optionally correlated to a trace
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exemplar {
    /// Timestamp (nanoseconds since epoch)
    pub time_unix_nano: i64,
    /// Measured value
    pub value: f64,
    /// Trace ID of the recording span (hex, if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Span ID of the recording span (hex, if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    /// Attributes filtered out of the data point
    #[serde(default, skip_serializing_if = "Attributes::is_empty")]
    pub filtered_attributes: Attributes,
}

/// Aggregate detail of a distribution data point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Distribution {
    /// Explicit-bounds histogram
    Histogram {
        /// Number of recorded values
        count: u64,
        /// Sum of recorded values (if reported)
        sum: Option<f64>,
        /// Minimum recorded value (if reported)
        min: Option<f64>,
        /// Maximum recorded value (if reported)
        max: Option<f64>,
        /// Per-bucket counts (len = `explicit_bounds.len() + 1`)
        bucket_counts: Vec<u64>,
        /// Upper bucket bounds
        explicit_bounds: Vec<f64>,
    },
    /// Base-2 exponentially scaled histogram
    ExponentialHistogram {
        /// Number of recorded values
        count: u64,
        /// Sum of recorded values (if reported)
        sum: Option<f64>,
        /// Minimum recorded value (if reported)
        min: Option<f64>,
        /// Maximum recorded value (if reported)
        max: Option<f64>,
        /// Scale factor: bucket base = 2^(2^-scale)
        scale: i32,
        /// Count of values exactly at zero (within `zero_threshold`)
        zero_count: u64,
        /// Width of the zero region
        zero_threshold: f64,
        /// Offset of the first positive bucket
        positive_offset: i32,
        /// Positive bucket counts
        positive_bucket_counts: Vec<u64>,
        /// Offset of the first negative bucket
        negative_offset: i32,
        /// Negative bucket counts
        negative_bucket_counts: Vec<u64>,
    },
    /// Pre-computed quantile summary
    Summary {
        /// Number of recorded values
        count: u64,
        /// Sum of recorded values
        sum: f64,
        /// Values at requested quantiles
        quantile_values: Vec<QuantileValue>,
    },
}

/// Represents a metric data point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDataPoint {
    /// Timestamp (nanoseconds since epoch)
    pub time_unix_nano: i64,
    /// Start time for cumulative metrics (nanoseconds since epoch)
    pub start_time_unix_nano: Option<i64>,
    /// Numeric value (for distributions: sum, or count when sum is absent)
    pub value: f64,
    /// Data point attributes
    pub attributes: Attributes,
    /// Aggregate detail for histogram/exponential histogram/summary points
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution: Option<Distribution>,
    /// Example measurements correlated to traces
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exemplars: Vec<Exemplar>,
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
            distribution: None,
            exemplars: Vec::new(),
        }
    }

    /// Attach distribution detail.
    #[must_use]
    pub fn with_distribution(mut self, distribution: Option<Distribution>) -> Self {
        self.distribution = distribution;
        self
    }

    /// Attach exemplars.
    #[must_use]
    pub fn with_exemplars(mut self, exemplars: Vec<Exemplar>) -> Self {
        self.exemplars = exemplars;
        self
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
    /// Whether a Sum metric is monotonic (`None` for other types)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_monotonic: Option<bool>,
    /// Full resource attributes of the producer
    #[serde(default, skip_serializing_if = "Attributes::is_empty")]
    pub resource_attributes: Attributes,
    /// Instrumentation scope that produced the metric
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<InstrumentationScope>,
}

impl Metric {
    /// Build a metric from its component fields.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
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
            is_monotonic: None,
            resource_attributes: Attributes::new(),
            scope: None,
        }
    }

    /// Create a gauge metric
    #[must_use]
    pub fn gauge(
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
    pub fn counter(
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

    /// Set whether a Sum metric is monotonic.
    #[must_use]
    pub const fn with_is_monotonic(mut self, is_monotonic: Option<bool>) -> Self {
        self.is_monotonic = is_monotonic;
        self
    }

    /// Attach the full resource attributes.
    #[must_use]
    pub fn with_resource_attributes(mut self, resource_attributes: Attributes) -> Self {
        self.resource_attributes = resource_attributes;
        self
    }

    /// Attach the instrumentation scope.
    #[must_use]
    pub fn with_scope(mut self, scope: Option<InstrumentationScope>) -> Self {
        self.scope = scope;
        self
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
    fn test_metric_deserializes_pre_v1_json() {
        // Stored JSON written before distribution/exemplars/is_monotonic existed.
        let json = r#"{
            "name": "m", "description": null, "unit": null,
            "metric_type": "SUM", "temporality": "CUMULATIVE",
            "data_points": [{
                "time_unix_nano": 1, "start_time_unix_nano": null,
                "value": 2.0, "attributes": {}
            }],
            "service_name": "svc"
        }"#;
        let metric: Metric = serde_json::from_str(json).unwrap();
        assert_eq!(metric.is_monotonic, None);
        assert_eq!(metric.data_points[0].distribution, None);
        assert!(metric.data_points[0].exemplars.is_empty());
    }

    #[test]
    fn test_metric_histogram_distribution_serde() {
        let dp = MetricDataPoint::new(1_000, None, 15.0, Attributes::new())
            .with_distribution(Some(Distribution::Histogram {
                count: 4,
                sum: Some(15.0),
                min: Some(1.0),
                max: Some(9.0),
                bucket_counts: vec![1, 2, 1],
                explicit_bounds: vec![2.5, 5.0],
            }))
            .with_exemplars(vec![Exemplar {
                time_unix_nano: 999,
                value: 9.0,
                trace_id: Some("abc123".to_string()),
                span_id: Some("def456".to_string()),
                filtered_attributes: Attributes::new(),
            }]);
        let metric = Metric::new(
            "latency".to_string(),
            None,
            Some("ms".to_string()),
            MetricType::Histogram,
            AggregationTemporality::Delta,
            vec![dp],
            Some("svc".to_string()),
        );

        let json = serde_json::to_string(&metric).unwrap();
        assert!(json.contains(r#""kind":"HISTOGRAM""#));
        let deserialized: Metric = serde_json::from_str(&json).unwrap();
        assert_eq!(metric, deserialized);
    }

    #[test]
    fn test_metric_exponential_histogram_distribution_serde() {
        let dp = MetricDataPoint::new(1_000, None, 20.0, Attributes::new()).with_distribution(
            Some(Distribution::ExponentialHistogram {
                count: 5,
                sum: Some(20.0),
                min: None,
                max: None,
                scale: 2,
                zero_count: 1,
                zero_threshold: 0.0,
                positive_offset: -1,
                positive_bucket_counts: vec![2, 2],
                negative_offset: 0,
                negative_bucket_counts: vec![],
            }),
        );
        let metric = Metric::new(
            "latency".to_string(),
            None,
            None,
            MetricType::ExponentialHistogram,
            AggregationTemporality::Cumulative,
            vec![dp],
            None,
        );

        let json = serde_json::to_string(&metric).unwrap();
        assert!(json.contains(r#""metric_type":"EXPONENTIAL_HISTOGRAM""#));
        let deserialized: Metric = serde_json::from_str(&json).unwrap();
        assert_eq!(metric, deserialized);
    }

    #[test]
    fn test_metric_summary_distribution_serde() {
        let dp = MetricDataPoint::new(1_000, None, 30.0, Attributes::new()).with_distribution(
            Some(Distribution::Summary {
                count: 10,
                sum: 30.0,
                quantile_values: vec![
                    QuantileValue {
                        quantile: 0.5,
                        value: 2.0,
                    },
                    QuantileValue {
                        quantile: 0.99,
                        value: 8.0,
                    },
                ],
            }),
        );
        let metric = Metric::new(
            "latency".to_string(),
            None,
            None,
            MetricType::Summary,
            AggregationTemporality::Unspecified,
            vec![dp],
            None,
        );

        let json = serde_json::to_string(&metric).unwrap();
        let deserialized: Metric = serde_json::from_str(&json).unwrap();
        assert_eq!(metric, deserialized);
    }

    #[test]
    fn test_metric_is_monotonic() {
        let metric =
            Metric::counter("reqs".to_string(), vec![], None).with_is_monotonic(Some(true));
        assert_eq!(metric.is_monotonic, Some(true));
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
