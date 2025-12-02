import { Link } from "@tanstack/react-router";
import type { TraceInfo } from "@/types";
import { formatRelativeTime } from "@/lib/formatters";
import { DurationBadge } from "@/components/shared/duration-badge";

interface RecentTracesProps {
  traces: TraceInfo[];
}

export function RecentTraces({ traces }: RecentTracesProps) {
  if (traces.length === 0) {
    return (
      <div className="border border-border p-8 text-center">
        <p className="text-sm text-foreground/50">No recent traces</p>
      </div>
    );
  }

  return (
    <div className="border border-border">
      <div className="px-4 py-3 border-b border-border">
        <h3 className="text-sm font-mono">Recent Traces</h3>
      </div>

      <div className="divide-y divide-border">
        {traces.slice(0, 5).map((trace) => (
          <Link
            key={trace.trace_id}
            to="/traces/$traceId"
            params={{ traceId: trace.trace_id }}
            className="flex items-center justify-between p-3 hover:bg-card/50 transition-colors"
          >
            <div className="flex-1 min-w-0">
              <div className="font-mono text-sm truncate">
                {trace.service_name || "unknown"}
              </div>
              <div className="text-xs text-foreground/50">
                {trace.start_time ? formatRelativeTime(trace.start_time) : "-"}
              </div>
            </div>

            <div className="flex items-center gap-3">
              <DurationBadge durationMs={trace.duration_ms} />
              {trace.has_errors && (
                <span className="text-xs text-red-500">ERROR</span>
              )}
            </div>
          </Link>
        ))}
      </div>

      <div className="px-4 py-3 border-t border-border">
        <Link
          to="/traces"
          search={{ service: undefined }}
          className="text-xs text-foreground/50 hover:text-foreground transition-colors"
        >
          View all traces →
        </Link>
      </div>
    </div>
  );
}
