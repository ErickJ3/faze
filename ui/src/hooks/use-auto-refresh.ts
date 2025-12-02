import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { getSettings } from "@/lib/settings";

export function useAutoRefresh() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const settings = getSettings();

    if (!settings.autoRefresh) {
      return;
    }

    const interval = setInterval(() => {
      queryClient.invalidateQueries();
    }, settings.refreshInterval);

    return () => clearInterval(interval);
  }, [queryClient]);
}
