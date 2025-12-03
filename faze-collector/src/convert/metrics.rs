use crate::convert::{convert_attributes, convert_resource};
use crate::proto::opentelemetry::proto::metrics::v1::{
    HistogramDataPoint, Metric as OtlpMetric, NumberDataPoint, ResourceMetrics, SummaryDataPoint,
    metric,
};
use faze::models::metric::{
    AggregationTemporality, Metric as FazeMetric, MetricDataPoint, MetricType as FazeMetricType,
};

fn convert_metric(otlp_metric: OtlpMetric, service_name: Option<String>) -> Option<FazeMetric> {
    let name = otlp_metric.name;
    let description = Some(otlp_metric.description);
    let unit = Some(otlp_metric.unit);

    match otlp_metric.data {
        Some(metric::Data::Gauge(gauge)) => {
            let data_points = gauge
                .data_points
                .into_iter()
                .map(convert_number_data_point)
                .collect();

            Some(FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::Gauge,
                AggregationTemporality::Unspecified,
                data_points,
                service_name,
            ))
        }
        Some(metric::Data::Sum(sum)) => {
            let data_points = sum
                .data_points
                .into_iter()
                .map(convert_number_data_point)
                .collect();

            let temporality = convert_temporality(sum.aggregation_temporality);

            Some(FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::Sum,
                temporality,
                data_points,
                service_name,
            ))
        }
        Some(metric::Data::Histogram(hist)) => {
            let data_points = hist
                .data_points
                .into_iter()
                .map(convert_histogram_data_point)
                .collect();

            let temporality = convert_temporality(hist.aggregation_temporality);

            Some(FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::Histogram,
                temporality,
                data_points,
                service_name,
            ))
        }
        Some(metric::Data::Summary(summary)) => {
            let data_points = summary
                .data_points
                .into_iter()
                .map(convert_summary_data_point)
                .collect();

            Some(FazeMetric::new(
                name,
                description,
                unit,
                FazeMetricType::Summary,
                AggregationTemporality::Unspecified,
                data_points,
                service_name,
            ))
        }
        _ => None,
    }
}

pub fn convert_resource_metrics(resource_metrics: Vec<ResourceMetrics>) -> Vec<FazeMetric> {
    let mut faze_metrics = Vec::new();

    for rm in resource_metrics {
        let service_name = rm
            .resource
            .as_ref()
            .map(convert_resource)
            .and_then(|r| r.service_name().map(|s| s.to_string()));

        for sm in rm.scope_metrics {
            for metric in sm.metrics {
                if let Some(gm) = convert_metric(metric, service_name.clone()) {
                    faze_metrics.push(gm);
                }
            }
        }
    }

    faze_metrics
}

fn convert_number_data_point(dp: NumberDataPoint) -> MetricDataPoint {
    let value = match dp.value {
        Some(v) => match v {
            crate::proto::opentelemetry::proto::metrics::v1::number_data_point::Value::AsDouble(
                d,
            ) => d,
            crate::proto::opentelemetry::proto::metrics::v1::number_data_point::Value::AsInt(i) => {
                i as f64
            }
        },
        None => 0.0,
    };

    MetricDataPoint::new(
        dp.time_unix_nano as i64,
        Some(dp.start_time_unix_nano as i64),
        value,
        convert_attributes(&dp.attributes),
    )
}

fn convert_histogram_data_point(dp: HistogramDataPoint) -> MetricDataPoint {
    let value = dp.sum.unwrap_or(dp.count as f64);

    MetricDataPoint::new(
        dp.time_unix_nano as i64,
        Some(dp.start_time_unix_nano as i64),
        value,
        convert_attributes(&dp.attributes),
    )
}

fn convert_summary_data_point(dp: SummaryDataPoint) -> MetricDataPoint {
    MetricDataPoint::new(
        dp.time_unix_nano as i64,
        Some(dp.start_time_unix_nano as i64),
        dp.sum,
        convert_attributes(&dp.attributes),
    )
}

fn convert_temporality(t: i32) -> AggregationTemporality {
    match t {
        1 => AggregationTemporality::Delta,
        2 => AggregationTemporality::Cumulative,
        _ => AggregationTemporality::Unspecified,
    }
}
