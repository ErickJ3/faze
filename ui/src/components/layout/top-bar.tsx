import { useProject } from "@/hooks/api";

export function Topbar() {
  const { data: project } = useProject();

  const displayPath = project?.path || "~/my-app";
  const shortPath = displayPath.replace(/^\/home\/[^/]+/, "~");

  return (
    <header
      className="
        h-12 flex items-center px-4
        border-b border-border
        bg-background/80 backdrop-blur
      "
    >
      <div className="text-sm text-muted-foreground font-mono">
        project:{" "}
        <span className="text-foreground" title={displayPath}>
          {shortPath}
        </span>
      </div>

      <div className="ml-auto flex items-center gap-4">
        <div className="text-sm text-muted-foreground font-mono">Cltr+K</div>
      </div>
    </header>
  );
}
