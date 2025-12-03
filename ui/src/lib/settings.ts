const SETTINGS_KEY = "faze-settings";

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
  } catch (err) {
    console.error("Failed to save settings:", err);
  }
}
