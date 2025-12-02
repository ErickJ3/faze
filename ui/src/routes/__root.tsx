import { AppShell } from "@/components/layout/app-shell";
import { createRootRoute, Outlet } from "@tanstack/react-router";
import { useAutoRefresh } from "@/hooks/use-auto-refresh";
import { CommandPalette } from "@/components/shared/command-palette";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  useAutoRefresh();

  return (
    <>
      <CommandPalette />
      <AppShell>
        <Outlet />
      </AppShell>
    </>
  );
}
