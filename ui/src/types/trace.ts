import type { Attributes, InstrumentationScope } from "./common";

export type SpanKind =
  | "UNSPECIFIED"
  | "INTERNAL"
  | "SERVER"
  | "CLIENT"
  | "PRODUCER"
  | "CONSUMER";

export type StatusCode = "UNSET" | "OK" | "ERROR";

export interface Status {
  code: StatusCode;
  message?: string;
}

export interface SpanEvent {
  time_unix_nano: number;
  name: string;
  attributes?: Attributes;
  dropped_attributes_count?: number;
}

export interface SpanLink {
  trace_id: string;
  span_id: string;
  trace_state?: string;
  attributes?: Attributes;
  dropped_attributes_count?: number;
}

export interface Span {
  span_id: string;
  trace_id: string;
  parent_span_id?: string;
  name: string;
  kind: SpanKind;
  start_time_unix_nano: number;
  end_time_unix_nano: number;
  attributes: Attributes;
  status: Status;
  service_name?: string;
  events?: SpanEvent[];
  links?: SpanLink[];
  trace_state?: string;
  dropped_attributes_count?: number;
  dropped_events_count?: number;
  dropped_links_count?: number;
  resource_attributes?: Attributes;
  scope?: InstrumentationScope;
}

export interface Trace {
  trace_id: string;
  spans: Span[];
  service_name?: string;
}

export interface TraceInfo {
  trace_id: string;
  service_name?: string;
  duration_ms: number;
  span_count: number;
  has_errors: boolean;
  start_time?: number;
  root_span_name?: string;
  root_span_kind?: SpanKind;
}

export interface TraceListResponse {
  traces: TraceInfo[];
  total: number;
}

export interface TraceFilters {
  service?: string;
  min_duration?: number;
  max_duration?: number;
  limit?: number;
  offset?: number;
}
