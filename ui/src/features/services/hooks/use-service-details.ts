import { useTraces, useLogs, useMetrics } from "@/hooks/api";
import { useMemo } from "react";

export function useServiceDetails(serviceName: string) {
  const {
    data: tracesData,
    isLoading: tracesLoading,
    error: tracesError,
    refetch: refetchTraces,
  } = useTraces({
    service: serviceName,
    limit: 100,
  });

  const {
    data: logsData,
    isLoading: logsLoading,
    error: logsError,
    refetch: refetchLogs,
  } = useLogs({
    service: serviceName,
    limit: 50,
  });

  const {
    data: metricsData,
    isLoading: metricsLoading,
    error: metricsError,
    refetch: refetchMetrics,
  } = useMetrics({
    service: serviceName,
    limit: 10,
  });

  const traces = tracesData?.traces ?? [];
  const logs = logsData ?? [];
  const metrics = metricsData ?? [];

  const stats = useMemo(() => {
    const totalTraces = traces.length;
    const errorTraces = traces.filter((t) => t.has_errors).length;
    const avgDuration =
      traces.length > 0
        ? traces.reduce((sum, t) => sum + t.duration_ms, 0) / traces.length
        : 0;

    const p95Duration =
      traces.length > 0
        ? calculatePercentile(
            traces.map((t) => t.duration_ms),
            95,
          )
        : 0;

    const totalLogs = logs.length;
    const errorLogs = logs.filter(
      (l) =>
        l.severity_level &&
        (l.severity_level === "ERROR" || l.severity_level === "FATAL"),
    ).length;

    const totalMetrics = metrics.length;

    return {
      totalTraces,
      errorTraces,
      errorRate: totalTraces > 0 ? (errorTraces / totalTraces) * 100 : 0,
      avgDuration,
      p95Duration,
      totalLogs,
      errorLogs,
      totalMetrics,
    };
  }, [traces, logs, metrics]);

  const recentTraces = useMemo(() => {
    return traces.slice(0, 10);
  }, [traces]);

  const recentLogs = useMemo(() => {
    return logs.slice(0, 10);
  }, [logs]);

  const isLoading = tracesLoading || logsLoading || metricsLoading;
  const error = tracesError || logsError || metricsError;

  const refetch = () => {
    refetchTraces();
    refetchLogs();
    refetchMetrics();
  };

  return {
    stats,
    recentTraces,
    allTraces: traces,
    recentLogs,
    metrics,
    isLoading,
    error,
    refetch,
  };
}

function calculatePercentile(values: number[], percentile: number): number {
  if (values.length === 0) return 0;

  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.ceil((percentile / 100) * sorted.length) - 1;
  return sorted[index];
}
