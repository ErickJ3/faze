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

export function getSettings(): Settings {
  try {
    const stored = localStorage.getItem(SETTINGS_KEY);
    if (stored) {
      return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
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
