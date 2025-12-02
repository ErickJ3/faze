import { createFileRoute } from "@tanstack/react-router";
import { MetricsHeader } from "@/features/metrics/components/metrics-header";
import { MetricsGrid } from "@/features/metrics/components/metrics-grid";
import { MetricFilters } from "@/features/metrics/components/metric-filters";
import { useMetricsData } from "@/features/metrics/hooks/use-metrics-data";
import { LoadingState } from "@/components/shared/loading-state";
import { ErrorState } from "@/components/shared/error-state";

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

  if (isLoading) {
    return <LoadingState />;
  }

  if (error) {
    return (
      <ErrorState
        message={error instanceof Error ? error.message : "Unknown error"}
        onRetry={() => refetch()}
      />
    );
  }

  return (
    <div>
      <MetricsHeader count={metrics.length} />

      <MetricFilters
        selectedService={filters.service}
        services={services}
        onServiceChange={(value) => updateFilter("service", value)}
      />

      <MetricsGrid metrics={metrics} />
    </div>
  );
}
