import type { Attributes } from "./common";

export type SeverityLevel =
  | "Unspecified"
  | "Trace"
  | "Trace2"
  | "Trace3"
  | "Trace4"
  | "Debug"
  | "Debug2"
  | "Debug3"
  | "Debug4"
  | "Info"
  | "Info2"
  | "Info3"
  | "Info4"
  | "Warn"
  | "Warn2"
  | "Warn3"
  | "Warn4"
  | "Error"
  | "Error2"
  | "Error3"
  | "Error4"
  | "Fatal"
  | "Fatal2"
  | "Fatal3"
  | "Fatal4";

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
  limit?: number;
}
