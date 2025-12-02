import { apiClient } from "../client";
import type { Metric, MetricFilters } from "@/types";

interface MetricsResponse {
  metrics: Metric[];
}

export const metricsService = {
  getMetrics: async (filters?: MetricFilters): Promise<Metric[]> => {
    const { data } = await apiClient.get<MetricsResponse>("/metrics", {
      params: filters,
    });
    return data.metrics;
  },
};
