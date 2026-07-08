import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useRouterState } from "@tanstack/react-router";
import { getSettings, SETTINGS_CHANGED_EVENT } from "@/lib/settings";

/** Query-key roots each route actually reads, so a refresh tick only
 * invalidates data the current view depends on. */
function activeKeysForPath(path: string): string[] {
  if (path === "/") return ["stats", "traces"];
  if (path.startsWith("/traces/")) return ["trace", "logs"];
  if (path.startsWith("/traces")) return ["traces"];
  if (path.startsWith("/logs")) return ["logs"];
  if (path.startsWith("/metrics")) return ["metrics"];
  if (path.startsWith("/services/"))
    return ["services", "stats", "traces", "logs", "metrics"];
  if (path.startsWith("/services")) return ["services"];
  return [];
}

export function useAutoRefresh() {
  const queryClient = useQueryClient();
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  });

  useEffect(() => {
    let interval: ReturnType<typeof setInterval> | undefined;
    const keys = activeKeysForPath(pathname);

    const arm = () => {
      if (interval) {
        clearInterval(interval);
        interval = undefined;
      }

      const settings = getSettings();
      if (!settings.autoRefresh || keys.length === 0) {
        return;
      }

      interval = setInterval(() => {
        queryClient.invalidateQueries({
          predicate: (query) => keys.includes(query.queryKey[0] as string),
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
  }, [queryClient, pathname]);
}
