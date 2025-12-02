import type { TraceInfo } from "@/types";
import { formatDurationCompact } from "@/lib/formatters";
import { useMemo } from "react";
import {
  ChartContainer,
  ChartTooltip,
  type ChartConfig,
} from "@/components/ui/chart";
import { Bar, BarChart, XAxis, YAxis, Cell } from "recharts";
import { useNavigate } from "@tanstack/react-router";

interface ServiceChartProps {
  traces: TraceInfo[];
}

interface ChartDataPoint {
  name: string;
  duration: number;
  hasErrors: boolean;
  trace: TraceInfo;
}

export function ServiceChart({ traces }: ServiceChartProps) {
  const navigate = useNavigate();

  const chartData = useMemo((): ChartDataPoint[] => {
    if (traces.length === 0) return [];

    const sorted = [...traces]
      .filter((t) => t.start_time && t.duration_ms != null)
      .sort((a, b) => (a.start_time || 0) - (b.start_time || 0))
      .slice(-20);

    if (sorted.length === 0) return [];

    return sorted.map((trace, index) => ({
      name: `#${index + 1}`,
      duration: trace.duration_ms,
      hasErrors: trace.has_errors,
      trace,
    }));
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

  const chartConfig = {
    duration: {
      label: "Response Time",
    },
  } satisfies ChartConfig;

  const handleBarClick = (data: ChartDataPoint) => {
    if (data?.trace?.trace_id) {
      navigate({ to: `/traces/${data.trace.trace_id}` });
    }
  };

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

      <ChartContainer config={chartConfig} className="h-32 w-full">
        <BarChart
          data={chartData}
          onClick={(data) => {
            if (data && data.activePayload && data.activePayload[0]) {
              handleBarClick(data.activePayload[0].payload as ChartDataPoint);
            }
          }}
          className="cursor-pointer"
        >
          <XAxis dataKey="name" hide />
          <YAxis hide />
          <ChartTooltip
            content={({ active, payload }) => {
              if (!active || !payload || !payload.length) return null;

              const data = payload[0].payload as ChartDataPoint;
              const trace = data.trace;

              return (
                <div className="border border-border/50 bg-background rounded-lg px-3 py-2 text-xs shadow-xl">
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <div
                        className="w-2 h-2 rounded-sm"
                        style={{
                          backgroundColor: data.hasErrors
                            ? "hsl(var(--destructive))"
                            : "hsl(var(--chart-2))",
                        }}
                      />
                      <span className="font-medium">
                        {trace.root_span_name || "Unnamed"}
                      </span>
                    </div>
                    {trace.service_name && (
                      <div className="text-muted-foreground">
                        Service: {trace.service_name}
                      </div>
                    )}
                    <div className="text-muted-foreground">
                      Kind: {trace.root_span_kind || "Unknown"}
                    </div>
                    <div className="flex items-center justify-between gap-4 pt-1 border-t border-border/50">
                      <span className="text-muted-foreground">Duration:</span>
                      <span className="font-mono font-medium">
                        {formatDurationCompact(data.duration)}
                      </span>
                    </div>
                    <div className="text-xs text-muted-foreground/70 pt-1">
                      Click to view trace
                    </div>
                  </div>
                </div>
              );
            }}
          />
          <Bar
            dataKey="duration"
            radius={[2, 2, 0, 0]}
          >
            {chartData.map((entry, index) => (
              <Cell
                key={`cell-${String(index)}`}
                fill={
                  entry.hasErrors
                    ? "hsl(var(--destructive))"
                    : "hsl(var(--chart-2))"
                }
              />
            ))}
          </Bar>
        </BarChart>
      </ChartContainer>

      <div className="mt-2 text-xs text-foreground/50 text-center">
        Last {chartData.length} traces
      </div>
    </div>
  );
}
