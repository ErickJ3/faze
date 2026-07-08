import { formatDurationCompact } from "@/lib/formatters";
import { DURATION_FAST_MS, DURATION_SLOW_MS } from "@/lib/constants";

interface DurationBadgeProps {
  durationMs: number;
}

export function DurationBadge({ durationMs }: DurationBadgeProps) {
  const getColor = () => {
    if (durationMs < DURATION_FAST_MS) return "text-green-500 bg-green-500/10";
    if (durationMs < DURATION_SLOW_MS)
      return "text-yellow-500 bg-yellow-500/10";
    return "text-destructive bg-destructive/10";
  };

  return (
    <span className={`px-2 py-0.5 text-xs font-mono ${getColor()}`}>
      {formatDurationCompact(durationMs)}
    </span>
  );
}
