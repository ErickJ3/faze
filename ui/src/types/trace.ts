import type { Attributes } from "./common";

export type SpanKind =
  | "Unspecified"
  | "Internal"
  | "Server"
  | "Client"
  | "Producer"
  | "Consumer";

export type StatusCode = "Unset" | "Ok" | "Error";

export interface Status {
  code: StatusCode;
  message?: string;
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
