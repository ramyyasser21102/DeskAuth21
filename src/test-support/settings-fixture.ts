import type { Settings } from "@/lib/api";

/** What a fresh install reports on a system that can block screen capture. */
export const DEFAULT_SETTINGS: Settings = {
  auto_lock_minutes: 5,
  capture_protection: true,
  hide_codes: true,
  tray_shows_codes: false,
  lock_on_sleep: true,
  clipboard_clear_seconds: 15,
  shortcuts: { toggle_window: "CmdOrCtrl+Shift+A", lock_vault: null },
  capture_protection_supported: true,
  tray_menu_lists_accounts: false,
};
