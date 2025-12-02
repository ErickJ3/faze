import { ThemeToggle } from "./theme-toggle";

export function Topbar() {
  return (
    <header
      className="
        h-12 flex items-center px-4
        border-b border-border
        bg-background/80 backdrop-blur
      "
    >
      <div className="text-sm text-muted-foreground font-mono">
        project: <span className="text-foreground">~/my-app</span>
      </div>

      <div className="ml-auto flex items-center gap-4">
        <div className="text-xs text-muted-foreground font-mono">⌘K</div>
        <ThemeToggle />
      </div>
    </header>
  );
}
