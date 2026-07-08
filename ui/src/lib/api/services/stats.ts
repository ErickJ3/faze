import { apiClient } from "../client";
import type { StatsResponse } from "@/types";

export const statsService = {
  getStats: async (): Promise<StatsResponse> => {
    const { data } = await apiClient.get<StatsResponse>("/stats");
    return data;
  },
};
