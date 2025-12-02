import { apiClient } from "../client";
import type { Trace, TraceListResponse, TraceFilters } from "@/types";

export const tracesService = {
  getTraces: async (filters?: TraceFilters): Promise<TraceListResponse> => {
    const { data } = await apiClient.get<TraceListResponse>("/traces", {
      params: filters,
    });
    return data;
  },

  getTraceById: async (traceId: string): Promise<Trace> => {
    const { data } = await apiClient.get<Trace>(`/traces/${traceId}`);
    return data;
  },
};
