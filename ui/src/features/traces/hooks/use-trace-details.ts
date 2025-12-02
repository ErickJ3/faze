import { useTrace } from "@/hooks/api";

export function useTraceDetails(traceId: string) {
  const { data, isLoading, error, refetch } = useTrace(traceId);

  return {
    trace: data,
    isLoading,
    error,
    refetch,
  };
}
