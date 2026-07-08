import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import {
  getSettings,
  saveSettings,
  resetSettings,
  type Settings,
} from "@/lib/settings";
import { useToast } from "@/components/shared/toast-provider";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

export const Route = createFileRoute("/settings/")({
  component: SettingsPage,
});

function SettingsPage() {
  const [settings, setSettings] = useState<Settings>(getSettings());
  const [resetDialogOpen, setResetDialogOpen] = useState(false);
  const { showToast } = useToast();

  const handleSave = () => {
    saveSettings(settings);
    showToast("Settings saved", "success");
  };

  const handleReset = () => {
    resetSettings();
    setSettings(getSettings());
    setResetDialogOpen(false);
    showToast("Settings reset", "success");
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
          <h2 className="text-sm font-mono mb-4">Reset</h2>

          <div className="space-y-4">
            <div>
              <p className="text-sm text-foreground/50 mb-3">
                Restore all settings to their defaults
              </p>
              <button
                onClick={() => setResetDialogOpen(true)}
                className="text-sm border border-destructive text-destructive px-4 py-2 hover:bg-destructive/10 transition-colors"
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

      <Dialog open={resetDialogOpen} onOpenChange={setResetDialogOpen}>
        <DialogContent className="rounded-none shadow-none">
          <DialogHeader>
            <DialogTitle className="font-mono text-base">
              Reset settings?
            </DialogTitle>
            <DialogDescription>
              All settings will be restored to their defaults. This cannot be
              undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <button
              onClick={() => setResetDialogOpen(false)}
              className="text-sm border border-border px-4 py-2 hover:bg-card transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleReset}
              className="text-sm border border-destructive text-destructive px-4 py-2 hover:bg-destructive/10 transition-colors"
            >
              Reset
            </button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
