import type { Attributes } from "./common";

export type SeverityLevel =
  | "UNSPECIFIED"
  | "TRACE"
  | "TRACE2"
  | "TRACE3"
  | "TRACE4"
  | "DEBUG"
  | "DEBUG2"
  | "DEBUG3"
  | "DEBUG4"
  | "INFO"
  | "INFO2"
  | "INFO3"
  | "INFO4"
  | "WARN"
  | "WARN2"
  | "WARN3"
  | "WARN4"
  | "ERROR"
  | "ERROR2"
  | "ERROR3"
  | "ERROR4"
  | "FATAL"
  | "FATAL2"
  | "FATAL3"
  | "FATAL4";

export interface Log {
  time_unix_nano: number;
  severity_level: SeverityLevel;
  severity_text?: string;
  body: string;
  attributes: Attributes;
  trace_id?: string;
  span_id?: string;
  service_name?: string;
}

export interface LogFilters {
  service?: string;
  level?: string;
  trace_id?: string;
  limit?: number;
}
