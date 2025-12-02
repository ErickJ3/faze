import type { SeverityLevel, StatusCode } from "@/types";

export const severityColors: Record<string, string> = {
  TRACE: "text-foreground/40",
  DEBUG: "text-foreground/60",
  INFO: "text-foreground",
  WARN: "text-yellow-500",
  ERROR: "text-red-500",
  FATAL: "text-red-700",
};

export const severityBgColors: Record<string, string> = {
  TRACE: "bg-foreground/5",
  DEBUG: "bg-foreground/10",
  INFO: "bg-blue-500/10",
  WARN: "bg-yellow-500/10",
  ERROR: "bg-red-500/10",
  FATAL: "bg-red-700/20",
};

export const statusColors: Record<StatusCode, string> = {
  Unset: "text-foreground/40",
  Ok: "text-green-500",
  Error: "text-red-500",
};

export const statusBgColors: Record<StatusCode, string> = {
  Unset: "bg-foreground/5",
  Ok: "bg-green-500/10",
  Error: "bg-red-500/10",
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
