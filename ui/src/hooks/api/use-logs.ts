import { useQuery, keepPreviousData } from "@tanstack/react-query";
import { logsService } from "@/lib/api/services";
import type { LogFilters } from "@/types";

export function useLogs(filters?: LogFilters) {
  return useQuery({
    queryKey: ["logs", filters],
    queryFn: () => logsService.getLogs(filters),
    placeholderData: keepPreviousData,
  });
}
