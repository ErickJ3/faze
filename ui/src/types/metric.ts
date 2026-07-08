import type { Attributes, InstrumentationScope } from "./common";

export type MetricType =
  | "GAUGE"
  | "SUM"
  | "HISTOGRAM"
  | "SUMMARY"
  | "EXPONENTIAL_HISTOGRAM";

export type AggregationTemporality = "UNSPECIFIED" | "DELTA" | "CUMULATIVE";

export interface QuantileValue {
  quantile: number;
  value: number;
}

export interface Exemplar {
  time_unix_nano: number;
  value: number;
  trace_id?: string;
  span_id?: string;
  filtered_attributes?: Attributes;
}

export type Distribution =
  | {
      kind: "HISTOGRAM";
      count: number;
      sum?: number;
      min?: number;
      max?: number;
      bucket_counts: number[];
      explicit_bounds: number[];
    }
  | {
      kind: "EXPONENTIAL_HISTOGRAM";
      count: number;
      sum?: number;
      min?: number;
      max?: number;
      scale: number;
      zero_count: number;
      zero_threshold: number;
      positive_offset: number;
      positive_bucket_counts: number[];
      negative_offset: number;
      negative_bucket_counts: number[];
    }
  | {
      kind: "SUMMARY";
      count: number;
      sum: number;
      quantile_values: QuantileValue[];
    };

export interface MetricDataPoint {
  time_unix_nano: number;
  start_time_unix_nano?: number;
  value: number;
  attributes: Attributes;
  distribution?: Distribution;
  exemplars?: Exemplar[];
}

export interface Metric {
  name: string;
  description?: string;
  unit?: string;
  metric_type: MetricType;
  temporality: AggregationTemporality;
  data_points: MetricDataPoint[];
  service_name?: string;
  is_monotonic?: boolean;
  resource_attributes?: Attributes;
  scope?: InstrumentationScope;
}

export interface MetricFilters {
  service?: string;
  limit?: number;
}
