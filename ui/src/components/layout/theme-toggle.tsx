import { useState } from "react";
import { getTheme, toggleTheme } from "@/lib/theme";

export function ThemeToggle() {
  const [theme, setThemeState] = useState(getTheme());

  const handleToggle = () => {
    const next = toggleTheme();
    setThemeState(next);
  };

  return (
    <button
      onClick={handleToggle}
      className="text-xs border border-border px-2 py-1 hover:bg-card focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring transition-colors font-mono"
      title="Toggle theme"
      aria-label={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
    >
      <span aria-hidden="true">{theme === "dark" ? "☀" : "☾"}</span>
    </button>
  );
}
