const SETTINGS_KEY = "faze-settings";

export const SETTINGS_CHANGED_EVENT = "faze:settings-changed";

export interface Settings {
  autoRefresh: boolean;
  refreshInterval: number;
}

const DEFAULT_SETTINGS: Settings = {
  autoRefresh: false,
  refreshInterval: 30000,
};

const MIN_INTERVAL_MS = 5000;
const MAX_INTERVAL_MS = 300000;

/** Clamp a possibly-corrupt stored interval into the supported 5-300s range
 * so a bad value can never drive a sub-second polling loop. */
function clampInterval(value: unknown): number {
  const n = Number(value);
  if (!Number.isFinite(n)) return DEFAULT_SETTINGS.refreshInterval;
  return Math.min(MAX_INTERVAL_MS, Math.max(MIN_INTERVAL_MS, n));
}

export function getSettings(): Settings {
  try {
    const stored = localStorage.getItem(SETTINGS_KEY);
    if (stored) {
      const merged = { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
      return {
        autoRefresh: Boolean(merged.autoRefresh),
        refreshInterval: clampInterval(merged.refreshInterval),
      };
    }
  } catch (err) {
    console.error("Failed to load settings:", err);
  }
  return DEFAULT_SETTINGS;
}

export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    window.dispatchEvent(new Event(SETTINGS_CHANGED_EVENT));
  } catch (err) {
    console.error("Failed to save settings:", err);
  }
}

export function resetSettings(): void {
  try {
    localStorage.removeItem(SETTINGS_KEY);
    window.dispatchEvent(new Event(SETTINGS_CHANGED_EVENT));
  } catch (err) {
    console.error("Failed to reset settings:", err);
  }
}
