import { Link } from "@tanstack/react-router";
import type { TraceInfo } from "@/types";
import { formatRelativeTime, formatDurationCompact } from "@/lib/formatters";

interface ServiceTracesProps {
  traces: TraceInfo[];
  serviceName: string;
}

export function ServiceTraces({ traces, serviceName }: ServiceTracesProps) {
  if (traces.length === 0) {
    return (
      <div className="border border-border p-6">
        <h2 className="text-lg font-mono mb-4">Recent Traces</h2>
        <p className="text-sm text-foreground/50">
          No traces found for this service
        </p>
      </div>
    );
  }

  return (
    <div className="border border-border">
      <div className="flex items-center justify-between p-4 border-b border-border">
        <h2 className="text-lg font-mono">Recent Traces</h2>
        <Link
          to="/traces"
          search={{ service: serviceName }}
          className="text-xs text-foreground/50 hover:text-foreground transition-colors"
        >
          View all →
        </Link>
      </div>

      <div className="divide-y divide-border">
        {traces.map((trace) => (
          <Link
            key={trace.trace_id}
            to="/traces/$traceId"
            params={{ traceId: trace.trace_id }}
            className="block p-4 hover:bg-card/50 transition-colors"
          >
            <div className="flex items-start justify-between gap-4">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-sm font-mono truncate">
                    {trace.root_span_name || "unknown"}
                  </span>
                  {trace.has_errors && (
                    <span className="text-xs px-2 py-0.5 bg-red-500/10 text-red-500">
                      ERROR
                    </span>
                  )}
                </div>
                <div className="text-xs text-foreground/50 font-mono">
                  {trace.trace_id.substring(0, 16)}...
                </div>
              </div>

              <div className="flex items-center gap-4 text-right">
                <div>
                  <div className="text-sm font-mono">
                    {formatDurationCompact(trace.duration_ms)}
                  </div>
                  <div className="text-xs text-foreground/50">
                    {trace.span_count} spans
                  </div>
                </div>
                <div className="text-xs text-foreground/50 w-20">
                  {trace.start_time
                    ? formatRelativeTime(trace.start_time)
                    : "-"}
                </div>
              </div>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
