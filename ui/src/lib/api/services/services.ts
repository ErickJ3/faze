import { apiClient } from "../client";
import type { ServicesResponse } from "@/types";

export const servicesService = {
  getServices: async (): Promise<string[]> => {
    const { data } = await apiClient.get<ServicesResponse>("/services");
    return data.services;
  },
};
