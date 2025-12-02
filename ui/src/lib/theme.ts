const THEME_KEY = "glint-theme";

export type Theme = "light" | "dark";

export function getTheme(): Theme {
  if (typeof window === "undefined") return "dark";

  try {
    const stored = localStorage.getItem(THEME_KEY);
    if (stored === "light" || stored === "dark") {
      return stored;
    }
  } catch (err) {
    console.error("Failed to load theme:", err);
  }
  return "dark";
}

export function setTheme(theme: Theme): void {
  try {
    localStorage.setItem(THEME_KEY, theme);
    applyTheme(theme);
  } catch (err) {
    console.error("Failed to save theme:", err);
  }
}

export function applyTheme(theme: Theme): void {
  if (theme === "dark") {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
}

export function toggleTheme(): Theme {
  const current = getTheme();
  const next: Theme = current === "dark" ? "light" : "dark";
  setTheme(next);
  return next;
}
