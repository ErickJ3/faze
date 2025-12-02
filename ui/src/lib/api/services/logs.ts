import { apiClient } from "../client";
import type { Log, LogFilters } from "@/types";

export const logsService = {
  getLogs: async (filters?: LogFilters): Promise<Log[]> => {
    const { data } = await apiClient.get<Log[]>("/logs", {
      params: filters,
    });
    return data;
  },
};
