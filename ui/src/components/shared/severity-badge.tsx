import type { SeverityLevel } from "@/types";
import { getSeverityColor, getSeverityBgColor } from "@/lib/colors";
import { SEVERITY_LEVEL_MAP } from "@/lib/constants";

interface SeverityBadgeProps {
  level: SeverityLevel;
}

export function SeverityBadge({ level }: SeverityBadgeProps) {
  const displayLevel = SEVERITY_LEVEL_MAP[level];

  return (
    <span
      className={`px-2 py-0.5 text-xs font-mono ${getSeverityColor(level)} ${getSeverityBgColor(level)}`}
    >
      {displayLevel}
    </span>
  );
}
