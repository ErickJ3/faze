import type { SeverityLevel } from "@/types";

/// Duration thresholds (ms) for fast/medium/slow badge coloring.
export const DURATION_FAST_MS = 100;
export const DURATION_SLOW_MS = 500;

/// Collapses the OTLP fine-grained levels (e.g. ERROR2..4) into display groups.
export const SEVERITY_LEVEL_MAP: Record<SeverityLevel, string> = {
  UNSPECIFIED: "UNSPECIFIED",
  TRACE: "TRACE",
  TRACE2: "TRACE",
  TRACE3: "TRACE",
  TRACE4: "TRACE",
  DEBUG: "DEBUG",
  DEBUG2: "DEBUG",
  DEBUG3: "DEBUG",
  DEBUG4: "DEBUG",
  INFO: "INFO",
  INFO2: "INFO",
  INFO3: "INFO",
  INFO4: "INFO",
  WARN: "WARN",
  WARN2: "WARN",
  WARN3: "WARN",
  WARN4: "WARN",
  ERROR: "ERROR",
  ERROR2: "ERROR",
  ERROR3: "ERROR",
  ERROR4: "ERROR",
  FATAL: "FATAL",
  FATAL2: "FATAL",
  FATAL3: "FATAL",
  FATAL4: "FATAL",
};
