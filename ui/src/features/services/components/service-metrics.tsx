import { Link } from "@tanstack/react-router";
import type { Metric } from "@/types";
import { formatNumber } from "@/lib/formatters";

interface ServiceMetricsProps {
  metrics: Metric[];
  serviceName: string;
}

export function ServiceMetrics({ metrics, serviceName }: ServiceMetricsProps) {
  if (metrics.length === 0) {
    return (
      <div className="border border-border p-6">
        <h2 className="text-lg font-mono mb-4">Metrics</h2>
        <p className="text-sm text-foreground/50">
          No metrics found for this service
        </p>
      </div>
    );
  }

  return (
    <div className="border border-border">
      <div className="flex items-center justify-between p-4 border-b border-border">
        <h2 className="text-lg font-mono">Metrics</h2>
        <Link
          to="/metrics"
          search={{ service: serviceName }}
          className="text-xs text-foreground/50 hover:text-foreground transition-colors"
        >
          View all →
        </Link>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-3 gap-4 p-4">
        {metrics.map((metric, index) => {
          const latestValue =
            metric.data_points[metric.data_points.length - 1]?.value;

          return (
            <div key={index} className="border border-border p-3">
              <div
                className="text-xs text-foreground/50 mb-1 truncate"
                title={metric.name}
              >
                {metric.name}
              </div>
              <div className="text-lg font-mono">
                {latestValue != null ? formatNumber(latestValue) : "-"}
              </div>
              <div className="text-xs text-foreground/30 mt-1 capitalize">
                {metric.metric_type.toLowerCase()}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
