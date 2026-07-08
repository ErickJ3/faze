import { useStats, useTraces } from "@/hooks/api";

export function useDashboardData() {
  const stats = useStats();
  const traces = useTraces({ limit: 5 });

  return {
    stats: stats.data,
    recentTraces: traces.data?.traces ?? [],
    isLoading: stats.isLoading || traces.isLoading,
    error: stats.error ?? traces.error,
    refetch: () => {
      void stats.refetch();
      void traces.refetch();
    },
  };
}
