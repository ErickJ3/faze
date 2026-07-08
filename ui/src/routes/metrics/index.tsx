import { createFileRoute } from "@tanstack/react-router";
import { MetricsHeader } from "@/features/metrics/components/metrics-header";
import { MetricsGrid } from "@/features/metrics/components/metrics-grid";
import { MetricFilters } from "@/features/metrics/components/metric-filters";
import { useMetricsData } from "@/features/metrics/hooks/use-metrics-data";
import { QueryBoundary } from "@/components/shared/query-boundary";
import { GridPageSkeleton } from "@/components/shared/page-skeletons";

export const Route = createFileRoute("/metrics/")({
  component: MetricsPage,
});

function MetricsPage() {
  const {
    metrics,
    services,
    filters,
    updateFilter,
    isLoading,
    error,
    refetch,
  } = useMetricsData();

  return (
    <QueryBoundary
      isLoading={isLoading}
      error={error}
      onRetry={() => refetch()}
      errorTitle="Couldn't load metrics"
      loadingFallback={<GridPageSkeleton filters={1} />}
    >
      <div>
        <MetricsHeader count={metrics.length} />

        <MetricFilters
          selectedService={filters.service}
          services={services}
          onServiceChange={(value) => updateFilter("service", value)}
        />

        <MetricsGrid metrics={metrics} />
      </div>
    </QueryBoundary>
  );
}
