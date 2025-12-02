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
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        setOpen((open) => !open);
      }

      if (open) {
        if (e.key === "h" && !e.metaKey && !e.ctrlKey) {
          const input = document.querySelector(
            "[cmdk-input]",
          ) as HTMLInputElement;
          if (document.activeElement !== input) {
            e.preventDefault();
            navigate({ to: "/", search: {} });
            setOpen(false);
          }
        }
        if (e.key === "j" && !e.metaKey && !e.ctrlKey) {
          const input = document.querySelector(
            "[cmdk-input]",
          ) as HTMLInputElement;
          if (document.activeElement !== input) {
            e.preventDefault();
            const items = document.querySelectorAll("[cmdk-item]");
            const current = Array.from(items).findIndex(
              (item) => item.getAttribute("data-selected") === "true",
            );
            if (current < items.length - 1) {
              (items[current + 1] as HTMLElement).click();
            }
          }
        }
        if (e.key === "k" && !e.metaKey && !e.ctrlKey) {
          const input = document.querySelector(
            "[cmdk-input]",
          ) as HTMLInputElement;
          if (document.activeElement !== input) {
            e.preventDefault();
            const items = document.querySelectorAll("[cmdk-item]");
            const current = Array.from(items).findIndex(
              (item) => item.getAttribute("data-selected") === "true",
            );
            if (current > 0) {
              (items[current - 1] as HTMLElement).click();
            }
          }
        }
      }
    };

    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, [navigate, open]);

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
