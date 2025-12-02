import { createFileRoute } from "@tanstack/react-router";
import { StatCard } from "@/features/dashboard/components/stat-card";
import { RecentTraces } from "@/features/dashboard/components/recent-traces";
import { useDashboardData } from "@/features/dashboard/hooks/use-dashboard-data";
import { LoadingState } from "@/components/shared/loading-state";

export const Route = createFileRoute("/")({
  component: DashboardPage,
});

function DashboardPage() {
  const { traces, stats, isLoading } = useDashboardData();

  if (isLoading) {
    return <LoadingState />;
  }

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-mono mb-1">Dashboard</h1>
        <p className="text-sm text-foreground/50">
          Overview of your observability data
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <StatCard label="Services" value={stats.totalServices} />
        <StatCard label="Total Traces" value={stats.totalTraces} />
        <StatCard label="Error Rate" value={`${stats.errorRate}%`} />
        <StatCard label="Avg Duration" value={`${stats.avgDuration}ms`} />
      </div>

      <RecentTraces traces={traces} />
    </div>
  );
}
