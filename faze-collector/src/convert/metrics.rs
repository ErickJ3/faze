use crate::convert::{
    ResourceContext, convert_attributes, convert_resource_context, convert_scope, id_to_hex,
};
use crate::proto::opentelemetry::proto::metrics::v1::{
    Exemplar as OtlpExemplar, ExponentialHistogramDataPoint, HistogramDataPoint,
    Metric as OtlpMetric, NumberDataPoint, ResourceMetrics, SummaryDataPoint, exemplar, metric,
    number_data_point,
};
use faze::models::InstrumentationScope as FazeScope;
use faze::models::metric::{
    AggregationTemporality, Distribution, Exemplar, Metric as FazeMetric, MetricDataPoint,
    MetricType as FazeMetricType, QuantileValue,
};

fn convert_metric(
    otlp_metric: OtlpMetric,
    resource: &ResourceContext,
    scope: Option<&FazeScope>,
) -> Option<FazeMetric> {
    let name = otlp_metric.name;
    let description = Some(otlp_metric.description);
    let unit = Some(otlp_metric.unit);

    let metric = match otlp_metric.data {
        Some(metric::Data::Gauge(gauge)) => {
            let data_points = gauge
                .data_points
                .into_iter()
                .map(convert_number_data_point)
                .collect();

            FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::Gauge,
                AggregationTemporality::Unspecified,
                data_points,
                resource.service_name.clone(),
            )
        }
        Some(metric::Data::Sum(sum)) => {
            let data_points = sum
                .data_points
                .into_iter()
                .map(convert_number_data_point)
                .collect();

            let temporality = convert_temporality(sum.aggregation_temporality);

            FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::Sum,
                temporality,
                data_points,
                resource.service_name.clone(),
            )
            .with_is_monotonic(Some(sum.is_monotonic))
        }
        Some(metric::Data::Histogram(hist)) => {
            let data_points = hist
                .data_points
                .into_iter()
                .map(convert_histogram_data_point)
                .collect();

            let temporality = convert_temporality(hist.aggregation_temporality);

            FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::Histogram,
                temporality,
                data_points,
                resource.service_name.clone(),
            )
        }
        Some(metric::Data::ExponentialHistogram(hist)) => {
            let data_points = hist
                .data_points
                .into_iter()
                .map(convert_exponential_histogram_data_point)
                .collect();

            let temporality = convert_temporality(hist.aggregation_temporality);

            FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::ExponentialHistogram,
                temporality,
                data_points,
                resource.service_name.clone(),
            )
        }
        Some(metric::Data::Summary(summary)) => {
            let data_points = summary
                .data_points
                .into_iter()
                .map(convert_summary_data_point)
                .collect();

            FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::Summary,
                AggregationTemporality::Unspecified,
                data_points,
                resource.service_name.clone(),
            )
        }
        None => return None,
    };

    Some(
        metric
            .with_resource_attributes(resource.attributes.clone())
            .with_scope(scope.cloned()),
    )
}

/// Convert a batch of OTLP `ResourceMetrics` into internal `Metric`s.
#[must_use]
pub fn convert_resource_metrics(resource_metrics: Vec<ResourceMetrics>) -> Vec<FazeMetric> {
    let mut faze_metrics = Vec::new();

    for rm in resource_metrics {
        let resource = convert_resource_context(rm.resource.as_ref());

        for sm in rm.scope_metrics {
            let scope = convert_scope(sm.scope.as_ref());
            for metric in sm.metrics {
                if let Some(gm) = convert_metric(metric, &resource, scope.as_ref()) {
                    faze_metrics.push(gm);
                }
            }
        }
    }

    faze_metrics
}

#[allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]
fn convert_exemplars(exemplars: &[OtlpExemplar]) -> Vec<Exemplar> {
    exemplars
        .iter()
        .map(|e| {
            let value = match e.value {
                Some(exemplar::Value::AsDouble(d)) => d,
                Some(exemplar::Value::AsInt(i)) => i as f64,
                None => 0.0,
            };
            Exemplar {
                time_unix_nano: e.time_unix_nano as i64,
                value,
                trace_id: id_to_hex(&e.trace_id),
                span_id: id_to_hex(&e.span_id),
                filtered_attributes: convert_attributes(&e.filtered_attributes),
            }
        })
        .collect()
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
fn convert_number_data_point(dp: NumberDataPoint) -> MetricDataPoint {
    let value = match dp.value {
        Some(number_data_point::Value::AsDouble(d)) => d,
        Some(number_data_point::Value::AsInt(i)) => i as f64,
        None => 0.0,
    };

    MetricDataPoint::new(
        dp.time_unix_nano as i64,
        Some(dp.start_time_unix_nano as i64),
        value,
        convert_attributes(&dp.attributes),
    )
    .with_exemplars(convert_exemplars(&dp.exemplars))
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
fn convert_histogram_data_point(dp: HistogramDataPoint) -> MetricDataPoint {
    let value = dp.sum.unwrap_or(dp.count as f64);

    MetricDataPoint::new(
        dp.time_unix_nano as i64,
        Some(dp.start_time_unix_nano as i64),
        value,
        convert_attributes(&dp.attributes),
    )
    .with_distribution(Some(Distribution::Histogram {
        count: dp.count,
        sum: dp.sum,
        min: dp.min,
        max: dp.max,
        bucket_counts: dp.bucket_counts,
        explicit_bounds: dp.explicit_bounds,
    }))
    .with_exemplars(convert_exemplars(&dp.exemplars))
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
fn convert_exponential_histogram_data_point(dp: ExponentialHistogramDataPoint) -> MetricDataPoint {
    let value = dp.sum.unwrap_or(dp.count as f64);

    let (positive_offset, positive_bucket_counts) = dp
        .positive
        .map(|b| (b.offset, b.bucket_counts))
        .unwrap_or_default();
    let (negative_offset, negative_bucket_counts) = dp
        .negative
        .map(|b| (b.offset, b.bucket_counts))
        .unwrap_or_default();

    MetricDataPoint::new(
        dp.time_unix_nano as i64,
        Some(dp.start_time_unix_nano as i64),
        value,
        convert_attributes(&dp.attributes),
    )
    .with_distribution(Some(Distribution::ExponentialHistogram {
        count: dp.count,
        sum: dp.sum,
        min: dp.min,
        max: dp.max,
        scale: dp.scale,
        zero_count: dp.zero_count,
        zero_threshold: dp.zero_threshold,
        positive_offset,
        positive_bucket_counts,
        negative_offset,
        negative_bucket_counts,
    }))
    .with_exemplars(convert_exemplars(&dp.exemplars))
}

#[allow(clippy::needless_pass_by_value, clippy::cast_possible_wrap)]
fn convert_summary_data_point(dp: SummaryDataPoint) -> MetricDataPoint {
    let quantile_values = dp
        .quantile_values
        .iter()
        .map(|q| QuantileValue {
            quantile: q.quantile,
            value: q.value,
        })
        .collect();

    MetricDataPoint::new(
        dp.time_unix_nano as i64,
        Some(dp.start_time_unix_nano as i64),
        dp.sum,
        convert_attributes(&dp.attributes),
    )
    .with_distribution(Some(Distribution::Summary {
        count: dp.count,
        sum: dp.sum,
        quantile_values,
    }))
}

const fn convert_temporality(t: i32) -> AggregationTemporality {
    match t {
        1 => AggregationTemporality::Delta,
        2 => AggregationTemporality::Cumulative,
        _ => AggregationTemporality::Unspecified,
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::proto::metrics::v1::{
        ExponentialHistogram, Histogram, ScopeMetrics, Sum, Summary,
        exponential_histogram_data_point::Buckets, summary_data_point::ValueAtQuantile,
    };

    fn wrap_metric(data: metric::Data) -> Vec<ResourceMetrics> {
        vec![ResourceMetrics {
            resource: None,
            scope_metrics: vec![ScopeMetrics {
                scope: None,
                metrics: vec![OtlpMetric {
                    name: "m".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: vec![],
                    data: Some(data),
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }]
    }

    #[test]
    fn test_convert_sum_is_monotonic() {
        let metrics = convert_resource_metrics(wrap_metric(metric::Data::Sum(Sum {
            data_points: vec![],
            aggregation_temporality: 2,
            is_monotonic: true,
        })));
        assert_eq!(metrics[0].is_monotonic, Some(true));
        assert_eq!(metrics[0].temporality, AggregationTemporality::Cumulative);
    }

    #[test]
    fn test_convert_histogram_keeps_buckets() {
        let metrics = convert_resource_metrics(wrap_metric(metric::Data::Histogram(Histogram {
            data_points: vec![HistogramDataPoint {
                count: 4,
                sum: Some(15.0),
                min: Some(1.0),
                max: Some(9.0),
                bucket_counts: vec![1, 2, 1],
                explicit_bounds: vec![2.5, 5.0],
                time_unix_nano: 1_000,
                ..Default::default()
            }],
            aggregation_temporality: 1,
        })));

        let dp = &metrics[0].data_points[0];
        assert_eq!(dp.value, 15.0);
        assert_eq!(
            dp.distribution,
            Some(Distribution::Histogram {
                count: 4,
                sum: Some(15.0),
                min: Some(1.0),
                max: Some(9.0),
                bucket_counts: vec![1, 2, 1],
                explicit_bounds: vec![2.5, 5.0],
            })
        );
    }

    #[test]
    fn test_convert_exponential_histogram() {
        let metrics = convert_resource_metrics(wrap_metric(metric::Data::ExponentialHistogram(
            ExponentialHistogram {
                data_points: vec![ExponentialHistogramDataPoint {
                    count: 5,
                    sum: Some(20.0),
                    scale: 2,
                    zero_count: 1,
                    positive: Some(Buckets {
                        offset: -1,
                        bucket_counts: vec![2, 2],
                    }),
                    negative: None,
                    time_unix_nano: 1_000,
                    ..Default::default()
                }],
                aggregation_temporality: 2,
            },
        )));

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].metric_type, FazeMetricType::ExponentialHistogram);
        let dp = &metrics[0].data_points[0];
        assert_eq!(dp.value, 20.0);
        let Some(Distribution::ExponentialHistogram {
            scale,
            zero_count,
            positive_offset,
            ref positive_bucket_counts,
            ..
        }) = dp.distribution
        else {
            unreachable!("expected exponential histogram distribution");
        };
        assert_eq!(scale, 2);
        assert_eq!(zero_count, 1);
        assert_eq!(positive_offset, -1);
        assert_eq!(*positive_bucket_counts, vec![2, 2]);
    }

    #[test]
    fn test_convert_summary_keeps_quantiles() {
        let metrics = convert_resource_metrics(wrap_metric(metric::Data::Summary(Summary {
            data_points: vec![SummaryDataPoint {
                count: 10,
                sum: 30.0,
                quantile_values: vec![
                    ValueAtQuantile {
                        quantile: 0.5,
                        value: 2.0,
                    },
                    ValueAtQuantile {
                        quantile: 0.99,
                        value: 8.0,
                    },
                ],
                time_unix_nano: 1_000,
                ..Default::default()
            }],
        })));

        let dp = &metrics[0].data_points[0];
        assert_eq!(dp.value, 30.0);
        assert_eq!(
            dp.distribution,
            Some(Distribution::Summary {
                count: 10,
                sum: 30.0,
                quantile_values: vec![
                    QuantileValue {
                        quantile: 0.5,
                        value: 2.0
                    },
                    QuantileValue {
                        quantile: 0.99,
                        value: 8.0
                    },
                ],
            })
        );
    }

    #[test]
    fn test_convert_number_data_point_exemplars() {
        let metrics = convert_resource_metrics(wrap_metric(metric::Data::Sum(Sum {
            data_points: vec![NumberDataPoint {
                value: Some(number_data_point::Value::AsInt(3)),
                exemplars: vec![OtlpExemplar {
                    time_unix_nano: 500,
                    value: Some(exemplar::Value::AsDouble(3.5)),
                    trace_id: vec![0xab, 0xcd],
                    span_id: vec![0x12],
                    filtered_attributes: vec![],
                }],
                time_unix_nano: 1_000,
                ..Default::default()
            }],
            aggregation_temporality: 1,
            is_monotonic: false,
        })));

        let dp = &metrics[0].data_points[0];
        assert_eq!(dp.value, 3.0);
        assert_eq!(dp.exemplars.len(), 1);
        assert_eq!(dp.exemplars[0].value, 3.5);
        assert_eq!(dp.exemplars[0].trace_id, Some("abcd".to_string()));
        assert_eq!(dp.exemplars[0].span_id, Some("12".to_string()));
    }
}
