import { useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";

const ROUTES = [
  { path: "/", label: "Dashboard", keys: ["d", "dashboard", "home"] },
  { path: "/services", label: "Services", keys: ["s", "services"] },
  { path: "/traces", label: "Traces", keys: ["t", "traces"] },
  { path: "/logs", label: "Logs", keys: ["l", "logs"] },
  { path: "/metrics", label: "Metrics", keys: ["m", "metrics"] },
  { path: "/settings", label: "Settings", keys: ["settings", "config"] },
];

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    // cmdk handles arrow-key navigation and Enter internally; we only own the
    // Ctrl+K open/close toggle, so this listener registers once and stays stable.
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        setOpen((prev) => !prev);
      }
    };

    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, []);

  const handleSelect = (path: string) => {
    navigate({ to: path as any, search: {} as any });
    setOpen(false);
  };

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <CommandInput placeholder="Search for pages..." />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>
        <CommandGroup heading="Navigation">
          {ROUTES.map((route) => (
            <CommandItem
              key={route.path}
              onSelect={() => handleSelect(route.path)}
              className="cursor-pointer"
            >
              <span>{route.label}</span>
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}
