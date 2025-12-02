import type { Metric } from "@/types";
import { formatNumber } from "@/lib/formatters";
import { useMemo } from "react";

interface MetricCardProps {
  metric: Metric;
}

export function MetricCard({ metric }: MetricCardProps) {
  const latestValue = metric.data_points[metric.data_points.length - 1];
  const previousValue =
    metric.data_points.length > 1
      ? metric.data_points[metric.data_points.length - 2]
      : null;

  const trend = previousValue
    ? latestValue.value > previousValue.value
      ? "up"
      : latestValue.value < previousValue.value
        ? "down"
        : "stable"
    : null;

  const sparklineData = useMemo(() => {
    if (metric.data_points.length < 2) return null;

    const values = metric.data_points.slice(-10).map((dp) => dp.value);
    const max = Math.max(...values);
    const min = Math.min(...values);
    const range = max - min || 1;

    return values.map((value, index) => ({
      x: (index / (values.length - 1)) * 100,
      y: 100 - ((value - min) / range) * 100,
    }));
  }, [metric.data_points]);

  return (
    <div className="border border-border p-4 bg-card hover:bg-card/80 transition-colors">
      <div className="mb-3">
        <div className="font-mono text-sm mb-1 truncate" title={metric.name}>
          {metric.name}
        </div>
        {metric.description && (
          <div
            className="text-xs text-foreground/50 truncate"
            title={metric.description}
          >
            {metric.description}
          </div>
        )}
      </div>

      <div className="flex items-end justify-between mb-3">
        <div>
          <div className="text-2xl font-mono">
            {formatNumber(latestValue.value)}
          </div>
          {metric.unit && (
            <div className="text-xs text-foreground/50">{metric.unit}</div>
          )}
        </div>

        {trend && (
          <div
            className={`text-xs font-mono ${
              trend === "up"
                ? "text-green-500"
                : trend === "down"
                  ? "text-red-500"
                  : "text-foreground/50"
            }`}
          >
            {trend === "up" ? "↑" : trend === "down" ? "↓" : "→"}
          </div>
        )}
      </div>

      {sparklineData && (
        <div className="h-8 -mx-1">
          <svg width="100%" height="100%" preserveAspectRatio="none">
            <polyline
              points={sparklineData.map((p) => `${p.x},${p.y}`).join(" ")}
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              className="text-primary"
              vectorEffect="non-scaling-stroke"
            />
          </svg>
        </div>
      )}

      <div className="flex items-center justify-between mt-2">
        {metric.service_name && (
          <div className="text-xs text-foreground/30 truncate">
            {metric.service_name}
          </div>
        )}
        <div className="text-xs text-foreground/30 capitalize">
          {metric.metric_type.toLowerCase()}
        </div>
      </div>
    </div>
  );
}
