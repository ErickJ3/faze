import { createFileRoute } from "@tanstack/react-router";
import { StatCard } from "@/features/dashboard/components/stat-card";
import { RecentTraces } from "@/features/dashboard/components/recent-traces";
import { DashboardSkeleton } from "@/features/dashboard/components/dashboard-skeleton";
import { TracesOverTimeChart } from "@/features/dashboard/components/traces-over-time-chart";
import { ServiceBreakdownChart } from "@/features/dashboard/components/service-breakdown-chart";
import { useDashboardData } from "@/features/dashboard/hooks/use-dashboard-data";
import { QueryBoundary } from "@/components/shared/query-boundary";
import { formatDurationCompact, formatNumber } from "@/lib/formatters";

export const Route = createFileRoute("/")({
  component: DashboardPage,
});

function EmptyDashboard() {
  return (
    <div className="border border-border p-12 text-center">
      <h2 className="text-lg font-mono mb-2">No data yet</h2>
      <p className="text-sm text-foreground/50 mb-4">
        Point your OpenTelemetry exporter at faze to start collecting traces,
        logs, and metrics.
      </p>
      <div className="inline-block text-left text-xs font-mono text-foreground/70 bg-card border border-border p-4">
        <div>OTLP gRPC: localhost:4317</div>
        <div>OTLP HTTP: localhost:4318</div>
      </div>
    </div>
  );
}

function DashboardPage() {
  const { stats, recentTraces, isLoading, error, refetch } =
    useDashboardData();

  const errorRate =
    stats && stats.traces.total > 0
      ? ((stats.traces.errors / stats.traces.total) * 100).toFixed(1)
      : "0.0";

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-mono mb-1">Dashboard</h1>
        <p className="text-sm text-foreground/50">
          Overview of your observability data
        </p>
      </div>

      <QueryBoundary
        isLoading={isLoading}
        error={error}
        onRetry={refetch}
        loadingFallback={<DashboardSkeleton />}
      >
        {stats && stats.traces.total === 0 ? (
          <EmptyDashboard />
        ) : (
          stats && (
            <>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
                <StatCard
                  label="Traces"
                  value={formatNumber(stats.traces.total)}
                  hint={`${formatNumber(stats.spans)} spans`}
                />
                <StatCard
                  label="Error Rate"
                  value={`${errorRate}%`}
                  hint={`${formatNumber(stats.traces.errors)} traces with errors`}
                />
                <StatCard
                  label="Avg Duration"
                  value={formatDurationCompact(stats.traces.avg_duration_ms)}
                />
                <StatCard
                  label="Services"
                  value={stats.services.length}
                  hint={`${formatNumber(stats.logs)} logs · ${formatNumber(stats.metrics)} metrics`}
                />
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-6">
                <TracesOverTimeChart activity={stats.activity} />
                <ServiceBreakdownChart services={stats.services} />
              </div>

              <RecentTraces traces={recentTraces} />
            </>
          )
        )}
      </QueryBoundary>
    </div>
  );
}
