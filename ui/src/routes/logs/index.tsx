import { createFileRoute } from "@tanstack/react-router";
import { LogsHeader } from "@/features/logs/components/logs-header";
import { LogsList } from "@/features/logs/components/logs-list";
import { LogFilters } from "@/features/logs/components/log-filters";
import { useLogsData } from "@/features/logs/hooks/use-logs-data";
import { QueryBoundary } from "@/components/shared/query-boundary";
import { Pagination } from "@/components/shared/pagination";

export const Route = createFileRoute("/logs/")({
  component: LogsPage,
});

function LogsPage() {
  const {
    logs,
    services,
    filters,
    pagination,
    updateFilter,
    isLoading,
    error,
    refetch,
  } = useLogsData();

  return (
    <QueryBoundary
      isLoading={isLoading}
      error={error}
      onRetry={() => refetch()}
    >
      <div>
        <LogsHeader count={pagination.totalItems} />

        <LogFilters
          selectedService={filters.service}
          services={services}
          onServiceChange={(value) => updateFilter("service", value)}
          selectedLevel={filters.level}
          onLevelChange={(value) => updateFilter("level", value)}
          searchQuery={filters.search}
          onSearchChange={(value) => updateFilter("search", value)}
        />

        <LogsList logs={logs} />

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
