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
import { Button } from "@/components/ui/button";
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
        <p className="text-sm text-muted-foreground">
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
                className="w-4 h-4 accent-primary cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
              <span className="text-sm">Enable auto-refresh</span>
            </label>

            {settings.autoRefresh && (
              <div>
                <label className="text-xs text-muted-foreground block mb-2">
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
                <p className="text-xs text-muted-foreground mt-1">
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
              <p className="text-sm text-muted-foreground mb-3">
                Restore all settings to their defaults
              </p>
              <Button
                variant="outline"
                onClick={() => setResetDialogOpen(true)}
                className="border-destructive text-destructive hover:bg-destructive/10 hover:text-destructive"
              >
                Reset Settings
              </Button>
            </div>
          </div>
        </section>

        <div className="flex gap-3">
          <Button onClick={handleSave}>Save Changes</Button>
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
            <Button variant="outline" onClick={() => setResetDialogOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleReset}>
              Reset
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
