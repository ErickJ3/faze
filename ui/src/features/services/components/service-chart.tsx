import type { TraceInfo } from "@/types";
import { formatDurationCompact } from "@/lib/formatters";
import { useMemo } from "react";

interface ServiceChartProps {
  traces: TraceInfo[];
}

export function ServiceChart({ traces }: ServiceChartProps) {
  const chartData = useMemo(() => {
    if (traces.length === 0) return [];

    const sorted = [...traces]
      .filter((t) => t.start_time && t.duration_ms != null)
      .sort((a, b) => (a.start_time || 0) - (b.start_time || 0))
      .slice(-20);

    if (sorted.length === 0) return [];

    const maxDuration = Math.max(...sorted.map((t) => t.duration_ms));
    const minDuration = Math.min(...sorted.map((t) => t.duration_ms));

    return sorted.map((trace, index) => {
      const range = maxDuration - minDuration;
      const normalizedHeight =
        range > 0 ? ((trace.duration_ms - minDuration) / range) * 100 : 100;

      return {
        trace,
        index,
        height: Math.max(normalizedHeight, 5),
      };
    });
  }, [traces]);

  const avgDuration = useMemo(() => {
    if (traces.length === 0) return 0;
    const validTraces = traces.filter((t) => t.duration_ms != null);
    if (validTraces.length === 0) return 0;
    return (
      validTraces.reduce((sum, t) => sum + t.duration_ms, 0) /
      validTraces.length
    );
  }, [traces]);

  const maxDuration = useMemo(() => {
    if (traces.length === 0) return 0;
    const validTraces = traces.filter((t) => t.duration_ms != null);
    if (validTraces.length === 0) return 0;
    return Math.max(...validTraces.map((t) => t.duration_ms));
  }, [traces]);

  if (chartData.length === 0) {
    return (
      <div className="border border-border p-6">
        <h2 className="text-lg font-mono mb-4">Response Time</h2>
        <p className="text-sm text-foreground/50">No data available</p>
      </div>
    );
  }

  return (
    <div className="border border-border p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-mono">Response Time</h2>
        <div className="flex items-center gap-4 text-xs">
          <div>
            <span className="text-foreground/50">Avg:</span>{" "}
            <span className="font-mono">
              {formatDurationCompact(avgDuration)}
            </span>
          </div>
          <div>
            <span className="text-foreground/50">Max:</span>{" "}
            <span className="font-mono">
              {formatDurationCompact(maxDuration)}
            </span>
          </div>
        </div>
      </div>

      <div className="h-32 flex items-end gap-1">
        {chartData.map((data) => (
          <div key={data.index} className="flex-1 relative group">
            <div
              className={`w-full transition-all ${
                data.trace.has_errors ? "bg-red-500" : "bg-green-500"
              } hover:opacity-80`}
              style={{
                height: `${data.height}%`,
              }}
            />
            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
              <div className="bg-background border border-border px-2 py-1 text-xs font-mono whitespace-nowrap">
                {formatDurationCompact(data.trace.duration_ms)}
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-2 text-xs text-foreground/50 text-center">
        Last {chartData.length} traces
      </div>
    </div>
  );
}
