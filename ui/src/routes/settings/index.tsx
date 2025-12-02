import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { getSettings, saveSettings, type Settings } from "@/lib/settings";
import { useToast } from "@/components/shared/toast-provider";
import { Input } from "@/components/ui/input";

export const Route = createFileRoute("/settings/")({
  component: SettingsPage,
});

function SettingsPage() {
  const [settings, setSettings] = useState<Settings>(getSettings());
  const { showToast } = useToast();

  const handleSave = () => {
    saveSettings(settings);
    showToast("Settings saved", "success");
    window.location.reload();
  };

  const handleReset = () => {
    const confirmed = confirm("Are you sure you want to reset all settings?");
    if (confirmed) {
      localStorage.clear();
      showToast("Settings reset", "success");
      setTimeout(() => window.location.reload(), 500);
    }
  };

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-mono mb-1">Settings</h1>
        <p className="text-sm text-foreground/50">
          Configure application preferences
        </p>
      </div>

      <div className="max-w-2xl space-y-6">
        <section className="border border-border p-6">
          <h2 className="text-sm font-mono mb-4">Auto Refresh</h2>

          <div className="space-y-4">
            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={settings.autoRefresh}
                onChange={(e) =>
                  setSettings({ ...settings, autoRefresh: e.target.checked })
                }
                className="w-4 h-4"
              />
              <span className="text-sm">Enable auto-refresh</span>
            </label>

            {settings.autoRefresh && (
              <div>
                <label className="text-xs text-foreground/50 block mb-2">
                  Refresh Interval (seconds)
                </label>
                <Input
                  type="number"
                  min="5"
                  max="300"
                  value={settings.refreshInterval / 1000}
                  onChange={(e) => {
                    const value = e.target.value;
                    if (value === "") {
                      setSettings({
                        ...settings,
                        refreshInterval: 5000,
                      });
                    } else {
                      const numValue = Math.max(
                        5,
                        Math.min(300, Number(value)),
                      );
                      setSettings({
                        ...settings,
                        refreshInterval: numValue * 1000,
                      });
                    }
                  }}
                  onFocus={(e) => e.target.select()}
                  className="w-32"
                />
                <p className="text-xs text-foreground/30 mt-1">
                  Data will refresh every {settings.refreshInterval / 1000}{" "}
                  seconds
                </p>
              </div>
            )}
          </div>
        </section>

        <section className="border border-border p-6">
          <h2 className="text-sm font-mono mb-4">Database</h2>

          <div className="space-y-4">
            <div>
              <p className="text-sm text-foreground/50 mb-3">
                Clear local database and reset application state
              </p>
              <button
                onClick={handleReset}
                className="text-sm border border-red-500 text-red-500 px-4 py-2 hover:bg-red-500/10 transition-colors"
              >
                Reset Settings
              </button>
            </div>
          </div>
        </section>

        <div className="flex gap-3">
          <button
            onClick={handleSave}
            className="text-sm border border-border px-4 py-2 hover:bg-card transition-colors"
          >
            Save Changes
          </button>
        </div>
      </div>
    </div>
  );
}
