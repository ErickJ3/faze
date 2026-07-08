import { useQuery } from "@tanstack/react-query";
import { statsService } from "@/lib/api/services";

export function useStats() {
  return useQuery({
    queryKey: ["stats"],
    queryFn: () => statsService.getStats(),
  });
}
