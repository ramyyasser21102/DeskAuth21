import type { Settings } from "@/lib/api";

/** What a fresh install reports on a system that can block screen capture. */
export const DEFAULT_SETTINGS: Settings = {
  auto_lock_minutes: 5,
  capture_protection: true,
  hide_codes: true,
  tray_shows_codes: false,
  capture_protection_supported: true,
};
