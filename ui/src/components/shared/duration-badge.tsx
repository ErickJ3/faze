import { formatDurationCompact } from "@/lib/formatters";

interface DurationBadgeProps {
  durationMs: number;
}

export function DurationBadge({ durationMs }: DurationBadgeProps) {
  const getColor = () => {
    if (durationMs < 100) return "text-green-500 bg-green-500/10";
    if (durationMs < 500) return "text-yellow-500 bg-yellow-500/10";
    return "text-red-500 bg-red-500/10";
  };

  return (
    <span className={`px-2 py-0.5 text-xs font-mono ${getColor()}`}>
      {formatDurationCompact(durationMs)}
    </span>
  );
}
