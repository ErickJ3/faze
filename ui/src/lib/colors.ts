import type { SeverityLevel, StatusCode } from "@/types";

export const severityColors: Record<string, string> = {
  TRACE: "text-sev-trace",
  DEBUG: "text-sev-debug",
  INFO: "text-sev-info",
  WARN: "text-sev-warn",
  ERROR: "text-sev-error",
  FATAL: "text-sev-fatal",
};

export const severityBgColors: Record<string, string> = {
  TRACE: "bg-sev-trace/10",
  DEBUG: "bg-sev-debug/10",
  INFO: "bg-sev-info/10",
  WARN: "bg-sev-warn/10",
  ERROR: "bg-sev-error/10",
  FATAL: "bg-sev-fatal/15",
};

export const statusColors: Record<StatusCode, string> = {
  UNSET: "text-status-unset",
  OK: "text-status-ok",
  ERROR: "text-status-error",
};

export const statusBgColors: Record<StatusCode, string> = {
  UNSET: "bg-status-unset/10",
  OK: "bg-status-ok/10",
  ERROR: "bg-status-error/10",
};

export function getSeverityColor(level: SeverityLevel): string {
  const levelStr = level.toString().toUpperCase();

  if (levelStr.includes("FATAL")) return severityColors.FATAL;
  if (levelStr.includes("ERROR")) return severityColors.ERROR;
  if (levelStr.includes("WARN")) return severityColors.WARN;
  if (levelStr.includes("INFO")) return severityColors.INFO;
  if (levelStr.includes("DEBUG")) return severityColors.DEBUG;
  if (levelStr.includes("TRACE")) return severityColors.TRACE;

  return severityColors.TRACE;
}

export function getSeverityBgColor(level: SeverityLevel): string {
  const levelStr = level.toString().toUpperCase();

  if (levelStr.includes("FATAL")) return severityBgColors.FATAL;
  if (levelStr.includes("ERROR")) return severityBgColors.ERROR;
  if (levelStr.includes("WARN")) return severityBgColors.WARN;
  if (levelStr.includes("INFO")) return severityBgColors.INFO;
  if (levelStr.includes("DEBUG")) return severityBgColors.DEBUG;
  if (levelStr.includes("TRACE")) return severityBgColors.TRACE;

  return severityBgColors.TRACE;
}
