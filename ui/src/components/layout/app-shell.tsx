import { Outlet } from "@tanstack/react-router";
import { MainSidebar } from "./main-sidebar";
import { Topbar } from "./top-bar";

interface AppShellProps {
  children?: React.ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  return (
    <div className="flex h-screen w-screen bg-background text-foreground">
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:border focus:border-border focus:bg-card focus:px-3 focus:py-2 focus:font-mono focus:text-sm"
      >
        Skip to content
      </a>
      <MainSidebar />

      <div className="flex flex-col flex-1 border-l border-border">
        <Topbar />

        <main
          id="main-content"
          tabIndex={-1}
          className="flex-1 overflow-auto p-4 outline-none"
        >
          {children || <Outlet />}
        </main>
      </div>
    </div>
  );
}
