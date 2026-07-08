import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { getSettings, SETTINGS_CHANGED_EVENT } from "@/lib/settings";

const DATA_KEYS = ["traces", "trace", "logs", "metrics", "services", "stats"];

export function useAutoRefresh() {
  const queryClient = useQueryClient();

  useEffect(() => {
    let interval: ReturnType<typeof setInterval> | undefined;

    const arm = () => {
      if (interval) {
        clearInterval(interval);
        interval = undefined;
      }

      const settings = getSettings();
      if (!settings.autoRefresh) {
        return;
      }

      interval = setInterval(() => {
        queryClient.invalidateQueries({
          predicate: (query) => DATA_KEYS.includes(query.queryKey[0] as string),
          refetchType: "active",
        });
      }, settings.refreshInterval);
    };

    arm();
    window.addEventListener(SETTINGS_CHANGED_EVENT, arm);

    return () => {
      if (interval) clearInterval(interval);
      window.removeEventListener(SETTINGS_CHANGED_EVENT, arm);
    };
  }, [queryClient]);
}
