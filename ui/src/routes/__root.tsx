import { AppShell } from "@/components/layout/app-shell";
import { createRootRoute, Outlet } from "@tanstack/react-router";
import { useAutoRefresh } from "@/hooks/use-auto-refresh";
import { CommandPalette } from "@/components/shared/command-palette";
import { Button } from "@/components/ui/button";

export const Route = createRootRoute({
  component: RootComponent,
  errorComponent: RootErrorComponent,
});

function RootErrorComponent({ error }: { error: Error }) {
  return (
    <div
      role="alert"
      className="flex h-screen w-screen flex-col items-center justify-center gap-4 bg-background p-6 text-center text-foreground"
    >
      <div className="space-y-1">
        <h1 className="font-mono text-lg">Something went wrong</h1>
        <p className="max-w-md font-mono text-sm text-muted-foreground">
          The dashboard hit an unexpected error and couldn't render this view.
        </p>
      </div>
      {error?.message ? (
        <pre className="max-w-lg overflow-auto border border-border bg-card p-3 text-left text-xs text-muted-foreground">
          {error.message}
        </pre>
      ) : null}
      <Button onClick={() => window.location.reload()}>Reload</Button>
    </div>
  );
}

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
