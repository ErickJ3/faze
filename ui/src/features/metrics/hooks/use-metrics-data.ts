import { useMetrics } from "@/hooks/api";
import { useServices } from "@/hooks/api";
import { useQueryStates, parseAsString } from "nuqs";

export function useMetricsData() {
  const [filters, setFilters] = useQueryStates({
    service: parseAsString,
  });

  const { data: servicesData } = useServices();
  const { data, isLoading, error, refetch } = useMetrics({
    limit: 100,
    service: filters.service ?? undefined,
  });

  const services = servicesData ?? [];
  const metrics = data ?? [];

  const updateFilter = (key: string, value: string | undefined) => {
    if (value === "all" || value === undefined) {
      setFilters({ [key]: null });
    } else {
      setFilters({ [key]: value });
    }
  };

  return {
    metrics,
    services,
    filters: {
      service: filters.service ?? undefined,
    },
    updateFilter,
    isLoading,
    error,
    refetch,
  };
}
