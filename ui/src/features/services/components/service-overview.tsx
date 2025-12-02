import { formatDurationCompact } from "@/lib/formatters";

interface ServiceOverviewProps {
  stats: {
    totalTraces: number;
    errorTraces: number;
    errorRate: number;
    avgDuration: number;
    p95Duration: number;
    totalLogs: number;
    errorLogs: number;
    totalMetrics: number;
  };
  serviceName: string;
}

export function ServiceOverview({ stats }: ServiceOverviewProps) {
  const healthStatus =
    stats.errorRate > 10
      ? "Unhealthy"
      : stats.errorRate > 5
        ? "Degraded"
        : "Healthy";
  const healthColor =
    stats.errorRate > 10
      ? "text-red-500"
      : stats.errorRate > 5
        ? "text-yellow-500"
        : "text-green-500";

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div className="border border-border p-4">
        <div className="text-xs text-foreground/50 mb-1">Health Status</div>
        <div className={`text-2xl font-mono ${healthColor}`}>
          {healthStatus}
        </div>
        <div className="text-xs text-foreground/30 mt-1">
          {stats.errorRate.toFixed(1)}% error rate
        </div>
      </div>

      <div className="border border-border p-4">
        <div className="text-xs text-foreground/50 mb-1">Total Traces</div>
        <div className="text-2xl font-mono">{stats.totalTraces}</div>
        <div className="text-xs text-foreground/30 mt-1">
          {stats.errorTraces} errors
        </div>
      </div>

      <div className="border border-border p-4">
        <div className="text-xs text-foreground/50 mb-1">Avg Duration</div>
        <div className="text-2xl font-mono">
          {formatDurationCompact(stats.avgDuration)}
        </div>
        <div className="text-xs text-foreground/30 mt-1">
          p95: {formatDurationCompact(stats.p95Duration)}
        </div>
      </div>

      <div className="border border-border p-4">
        <div className="text-xs text-foreground/50 mb-1">Logs</div>
        <div className="text-2xl font-mono">{stats.totalLogs}</div>
        <div className="text-xs text-foreground/30 mt-1">
          {stats.errorLogs} errors
        </div>
      </div>
    </div>
  );
}
