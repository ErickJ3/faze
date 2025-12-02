import type { SeverityLevel, SpanKind, StatusCode } from "@/types";

export const SEVERITY_LEVEL_MAP: Record<SeverityLevel, string> = {
  Unspecified: "UNSPECIFIED",
  Trace: "TRACE",
  Trace2: "TRACE",
  Trace3: "TRACE",
  Trace4: "TRACE",
  Debug: "DEBUG",
  Debug2: "DEBUG",
  Debug3: "DEBUG",
  Debug4: "DEBUG",
  Info: "INFO",
  Info2: "INFO",
  Info3: "INFO",
  Info4: "INFO",
  Warn: "WARN",
  Warn2: "WARN",
  Warn3: "WARN",
  Warn4: "WARN",
  Error: "ERROR",
  Error2: "ERROR",
  Error3: "ERROR",
  Error4: "ERROR",
  Fatal: "FATAL",
  Fatal2: "FATAL",
  Fatal3: "FATAL",
  Fatal4: "FATAL",
};

export const SPAN_KIND_MAP: Record<SpanKind, string> = {
  Unspecified: "UNSPECIFIED",
  Internal: "INTERNAL",
  Server: "SERVER",
  Client: "CLIENT",
  Producer: "PRODUCER",
  Consumer: "CONSUMER",
};

export const STATUS_CODE_MAP: Record<StatusCode, string> = {
  Unset: "UNSET",
  Ok: "OK",
  Error: "ERROR",
};

export function getSeverityColor(level: string): string {
  const upperLevel = level.toUpperCase();

  if (upperLevel === "FATAL" || upperLevel === "ERROR") {
    return "bg-red-500/10 text-red-500";
  }

  if (upperLevel === "WARN") {
    return "bg-yellow-500/10 text-yellow-500";
  }

  if (upperLevel === "INFO") {
    return "bg-blue-500/10 text-blue-500";
  }

  if (upperLevel === "DEBUG") {
    return "bg-purple-500/10 text-purple-500";
  }

  if (upperLevel === "TRACE") {
    return "bg-gray-500/10 text-gray-500";
  }

  return "bg-gray-500/10 text-gray-500";
}
