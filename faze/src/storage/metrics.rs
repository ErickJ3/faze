//! Metric persistence: inserts and listing.

use super::rows::{attrs_to_json, metric_from_row, opt_to_json, slice_to_json, to_json};
use super::{Result, Storage, service_filtered_query};
use crate::models::Metric;
use rusqlite::{Result as SqliteResult, params};

impl Storage {
    /// Insert a metric
    pub fn insert_metric(&self, metric: &Metric) -> Result<()> {
        self.insert_metrics(std::slice::from_ref(metric))
    }

    /// Insert multiple metrics atomically under a single transaction.
    pub fn insert_metrics(&self, metrics: &[Metric]) -> Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }
        self.with_tx(|tx| {
            let mut stmt = tx.prepare(
                "INSERT INTO metrics (
                    name, description, unit, metric_type, temporality,
                    time_unix_nano, start_time_unix_nano, value,
                    attributes, service_name,
                    is_monotonic, distribution, exemplars,
                    resource_attributes, scope
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            )?;
            for metric in metrics {
                let resource_attributes_json = attrs_to_json(&metric.resource_attributes)?;
                let scope_json = opt_to_json(metric.scope.as_ref())?;
                for data_point in &metric.data_points {
                    let attributes_json = to_json(&data_point.attributes)?;
                    let distribution_json = opt_to_json(data_point.distribution.as_ref())?;
                    let exemplars_json = slice_to_json(&data_point.exemplars)?;
                    stmt.execute(params![
                        &metric.name,
                        &metric.description,
                        &metric.unit,
                        metric.metric_type.as_db_str(),
                        metric.temporality.as_db_str(),
                        data_point.time_unix_nano,
                        data_point.start_time_unix_nano,
                        data_point.value,
                        attributes_json,
                        &metric.service_name,
                        metric.is_monotonic,
                        distribution_json,
                        exemplars_json,
                        &resource_attributes_json,
                        &scope_json,
                    ])?;
                }
            }
            Ok(())
        })
    }

    /// List metrics with optional service-name filter and result cap.
    #[allow(clippy::significant_drop_tightening)]
    pub fn list_metrics(
        &self,
        service_name: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Metric>> {
        let (query, params_vec) = service_filtered_query(
            "SELECT name, description, unit, metric_type, temporality,
                    time_unix_nano, start_time_unix_nano, value,
                    attributes, service_name,
                    is_monotonic, distribution, exemplars,
                    resource_attributes, scope
               FROM metrics",
            "time_unix_nano",
            service_name,
            limit,
        );

        let conn = self.lock()?;
        let mut stmt = conn.prepare(&query)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(AsRef::as_ref).collect();

        let metrics = stmt
            .query_map(&params_refs[..], metric_from_row)?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(metrics)
    }

    /// Get count of metrics
    pub fn count_metrics(&self) -> Result<i64> {
        self.count_rows("metrics")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AggregationTemporality, AttributeValue, Attributes, MetricDataPoint, MetricType,
    };

    #[test]
    fn test_metric_attributes_roundtrip_typed() {
        let storage = Storage::new_in_memory().unwrap();

        let mut attributes = Attributes::new();
        attributes.insert("string_key", "value");
        attributes.insert("int_key", 42i64);
        attributes.insert("double_key", 1.5f64);
        attributes.insert("bool_key", true);

        let metric = Metric::new(
            "test.metric".to_string(),
            None,
            None,
            MetricType::Gauge,
            AggregationTemporality::Unspecified,
            vec![MetricDataPoint::new(1_000_000_000, None, 1.0, attributes)],
            Some("test-service".to_string()),
        );

        storage.insert_metric(&metric).unwrap();
        let metrics = storage.list_metrics(None, None).unwrap();

        assert_eq!(metrics.len(), 1);
        let attrs = &metrics[0].data_points[0].attributes;
        assert_eq!(
            attrs.get("string_key"),
            Some(&AttributeValue::String("value".to_string()))
        );
        assert_eq!(attrs.get("int_key"), Some(&AttributeValue::Int(42)));
        assert_eq!(attrs.get("double_key"), Some(&AttributeValue::Double(1.5)));
        assert_eq!(attrs.get("bool_key"), Some(&AttributeValue::Bool(true)));
    }

    #[test]
    fn test_metric_full_fidelity_roundtrip() {
        use crate::models::{Distribution, Exemplar, InstrumentationScope, QuantileValue};

        let storage = Storage::new_in_memory().unwrap();
        let mut resource_attrs = Attributes::new();
        resource_attrs.insert("service.version", "2.0");

        let dp = MetricDataPoint::new(1_000_000_000, Some(900_000_000), 15.0, Attributes::new())
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
                trace_id: Some("trace1".to_string()),
                span_id: Some("span1".to_string()),
                filtered_attributes: Attributes::new(),
            }]);

        let metric = Metric::new(
            "latency".to_string(),
            Some("request latency".to_string()),
            Some("ms".to_string()),
            MetricType::Histogram,
            AggregationTemporality::Delta,
            vec![dp],
            Some("test-service".to_string()),
        )
        .with_is_monotonic(Some(false))
        .with_resource_attributes(resource_attrs)
        .with_scope(Some(InstrumentationScope::new(
            "test-lib".to_string(),
            Some("1.0".to_string()),
            Attributes::new(),
        )));

        storage.insert_metric(&metric).unwrap();
        let metrics = storage.list_metrics(None, None).unwrap();

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0], metric);

        // Summary distribution survives too.
        let summary_dp = MetricDataPoint::new(2_000_000_000, None, 30.0, Attributes::new())
            .with_distribution(Some(Distribution::Summary {
                count: 10,
                sum: 30.0,
                quantile_values: vec![QuantileValue {
                    quantile: 0.99,
                    value: 8.0,
                }],
            }));
        let summary = Metric::new(
            "latency.summary".to_string(),
            None,
            None,
            MetricType::Summary,
            AggregationTemporality::Unspecified,
            vec![summary_dp],
            None,
        );
        storage.insert_metric(&summary).unwrap();
        let metrics = storage.list_metrics(None, None).unwrap();
        assert!(metrics.contains(&summary));
    }

    #[test]
    fn test_count_metrics() {
        let storage = Storage::new_in_memory().unwrap();
        assert_eq!(storage.count_metrics().unwrap(), 0);
    }
}
