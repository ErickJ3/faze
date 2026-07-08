import { createFileRoute } from "@tanstack/react-router";
import { TracesHeader } from "@/features/traces/components/traces-header";
import { TracesTable } from "@/features/traces/components/traces-table";
import { TraceFilters } from "@/features/traces/components/trace-filters";
import { useTracesData } from "@/features/traces/hooks/use-traces-data";
import { QueryBoundary } from "@/components/shared/query-boundary";
import { Pagination } from "@/components/shared/pagination";

export const Route = createFileRoute("/traces/")({
  component: TracesPage,
  validateSearch: (search: Record<string, unknown>) => {
    return {
      service: search.service as string | undefined,
    };
  },
});

function TracesPage() {
  const {
    traces,
    total,
    services,
    filters,
    pagination,
    updateFilter,
    isLoading,
    error,
    refetch,
  } = useTracesData();

  return (
    <QueryBoundary
      isLoading={isLoading}
      error={error}
      onRetry={() => refetch()}
    >
      <div>
        <TracesHeader total={total} />

        <TraceFilters
          selectedService={filters.service}
          services={services}
          onServiceChange={(value) => updateFilter("service", value)}
          minDuration={filters.min_duration}
          maxDuration={filters.max_duration}
          onMinDurationChange={(value) => updateFilter("min_duration", value)}
          onMaxDurationChange={(value) => updateFilter("max_duration", value)}
          searchQuery={filters.search}
          onSearchChange={(value) => updateFilter("search", value)}
        />

        <TracesTable traces={traces} />

        <Pagination
          currentPage={pagination.currentPage}
          totalPages={pagination.totalPages}
          onPageChange={pagination.onPageChange}
          pageSize={pagination.pageSize}
          onPageSizeChange={pagination.onPageSizeChange}
          totalItems={pagination.totalItems}
        />
      </div>
    </QueryBoundary>
  );
}
