import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@/lib/api/client";
import type { ProjectInfo } from "@/types";

export function useProject() {
  return useQuery({
    queryKey: ["project"],
    queryFn: async () => {
      const response = await apiClient.get<ProjectInfo>("/project");
      return response.data;
    },
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
  });
}
