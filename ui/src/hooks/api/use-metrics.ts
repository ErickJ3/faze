import { useQuery, keepPreviousData } from "@tanstack/react-query";
import { metricsService } from "@/lib/api/services";
import type { MetricFilters } from "@/types";

export function useMetrics(filters?: MetricFilters) {
  return useQuery({
    queryKey: ["metrics", filters],
    queryFn: () => metricsService.getMetrics(filters),
    placeholderData: keepPreviousData,
  });
}
