import { useTraces } from "@/hooks/api";
import { useServices } from "@/hooks/api";

export function useDashboardData() {
  const { data: tracesData, isLoading: tracesLoading } = useTraces({
    limit: 10,
  });
  const { data: servicesData, isLoading: servicesLoading } = useServices();

  const traces = tracesData?.traces ?? [];
  const services = servicesData ?? [];
  const totalTraces = tracesData?.total ?? 0;

  const errorCount = traces.filter((t) => t.has_errors).length;
  const errorRate = traces.length > 0 ? (errorCount / traces.length) * 100 : 0;

  const avgDuration =
    traces.length > 0
      ? traces.reduce((acc, t) => acc + t.duration_ms, 0) / traces.length
      : 0;

  return {
    traces,
    services,
    stats: {
      totalServices: services.length,
      totalTraces,
      errorRate: errorRate.toFixed(1),
      avgDuration: avgDuration.toFixed(0),
    },
    isLoading: tracesLoading || servicesLoading,
  };
}
