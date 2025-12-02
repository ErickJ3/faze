import type { Metric } from "@/types";
import { MetricCard } from "./metric-card";

interface MetricsGridProps {
  metrics: Metric[];
}

export function MetricsGrid({ metrics }: MetricsGridProps) {
  if (metrics.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 border border-border">
        <div className="text-center">
          <p className="text-foreground/50 text-sm">No metrics found</p>
          <p className="text-foreground/30 text-xs mt-1">
            Adjust filters or start sending metrics
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {metrics.map((metric, index) => (
        <MetricCard key={`${metric.name}-${index}`} metric={metric} />
      ))}
    </div>
  );
}
