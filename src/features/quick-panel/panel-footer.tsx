import { Lock, Settings } from "lucide-react";
import { IconButton } from "@/components/icon-button";
import { Button } from "@/components/ui/button";
import { hideQuickPanel, lockVault, openAppPage } from "@/lib/api";

/** The three ways out of the panel: the app, its settings, and locking the vault. */
export function PanelFooter() {
  const lock = async () => {
    await lockVault();
    await hideQuickPanel();
  };
  return (
    <footer className="flex items-center justify-between gap-2 border-t p-2">
      <Button type="button" variant="ghost" size="sm" onClick={() => void openAppPage("codes")}>Open DeskAuth21</Button>
      <div className="flex items-center gap-1">
        <IconButton label="Open settings" variant="ghost" className="size-9" onClick={() => void openAppPage("settings")}><Settings /></IconButton>
        <IconButton label="Lock now" variant="ghost" className="size-9" onClick={() => void lock()}><Lock /></IconButton>
      </div>
    </footer>
  );
}
