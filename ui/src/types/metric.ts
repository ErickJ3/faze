import type { Attributes } from "./common";

export type MetricType = "Gauge" | "Sum" | "Histogram" | "Summary";

export type AggregationTemporality = "Unspecified" | "Delta" | "Cumulative";

export interface MetricDataPoint {
  time_unix_nano: number;
  start_time_unix_nano?: number;
  value: number;
  attributes: Attributes;
}

export interface Metric {
  name: string;
  description?: string;
  unit?: string;
  metric_type: MetricType;
  temporality: AggregationTemporality;
  data_points: MetricDataPoint[];
  service_name?: string;
}

export interface MetricFilters {
  service?: string;
  limit?: number;
}
