import { useState } from "react";
import { setShortcut, type ShortcutAction } from "@/lib/api";
import { usePrivacySettings } from "@/features/privacy/privacy-settings-provider";
import { ShortcutField } from "./shortcut-field";

const TOGGLE_DESCRIPTION = "Works from any app. Shows DA21, or hides it when it is already in front.";
const LOCK_DESCRIPTION = "Locks the vault right away from any app and clears the clipboard.";
const NO_SHORTCUT_NOTE = "Some desktops, such as Wayland on Linux, do not let apps listen for shortcuts.";

export function ShortcutSettings() {
  const { settings, applySaved } = usePrivacySettings();
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  if (!settings) return null;

  const choose = async (action: ShortcutAction, shortcut: string | null) => {
    try {
      applySaved(await setShortcut(action, shortcut));
      setErrorMessage(null);
    } catch (error) {
      setErrorMessage(String(error));
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <span className="text-sm font-medium">Keyboard shortcuts</span>
      <ShortcutField id="shortcut-toggle-window" label="Show or hide the window" description={TOGGLE_DESCRIPTION} shortcut={settings.shortcuts.toggle_window} onChoose={(shortcut) => void choose("toggle_window", shortcut)} />
      <ShortcutField id="shortcut-lock-vault" label="Lock the vault" description={LOCK_DESCRIPTION} shortcut={settings.shortcuts.lock_vault} onChoose={(shortcut) => void choose("lock_vault", shortcut)} />
      <p className="text-xs text-muted-foreground">{NO_SHORTCUT_NOTE}</p>
      {errorMessage && <p role="alert" className="text-xs text-destructive">{errorMessage}</p>}
    </div>
  );
}
