import { useQuery, keepPreviousData } from "@tanstack/react-query";
import { tracesService } from "@/lib/api/services";
import type { TraceFilters } from "@/types";

export function useTraces(filters?: TraceFilters) {
  return useQuery({
    queryKey: ["traces", filters],
    queryFn: () => tracesService.getTraces(filters),
    placeholderData: keepPreviousData,
  });
}

export function useTrace(traceId: string) {
  return useQuery({
    queryKey: ["trace", traceId],
    queryFn: () => tracesService.getTraceById(traceId),
    enabled: !!traceId,
  });
}
