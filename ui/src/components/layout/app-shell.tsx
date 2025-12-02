import { Outlet } from "@tanstack/react-router";
import { MainSidebar } from "./main-sidebar";
import { Topbar } from "./top-bar";

interface AppShellProps {
  children?: React.ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  return (
    <div className="flex h-screen w-screen bg-background text-foreground">
      <MainSidebar />

      <div className="flex flex-col flex-1 border-l border-border">
        <Topbar />

        <main className="flex-1 overflow-auto p-4">
          {children || <Outlet />}
        </main>
      </div>
    </div>
  );
}
