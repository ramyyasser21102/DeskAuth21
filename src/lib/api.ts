// The only module that talks to the Rust backend. Components call these typed
// functions and never `invoke` directly.
import { invoke } from "@tauri-apps/api/core";

export type VaultStatus = "uninitialized" | "locked" | "unlocked";

/** Ids match the `Vendor` enum in the Rust backend. */
export type VendorId =
  | "google_authenticator"
  | "microsoft_authenticator"
  | "authy"
  | "one_password"
  | "bitwarden"
  | "last_pass"
  | "duo_mobile"
  | "okta_verify"
  | "aegis"
  | "two_fas"
  | "proton_authenticator"
  | "ente_auth"
  | "free_otp"
  | "and_otp"
  | "raivo"
  | "other";

export interface VendorInfo {
  id: VendorId;
  name: string;
}

export const getVendors = () => invoke<VendorInfo[]>("list_vendors");

export interface CodeView {
  /** Stable across devices and reordering; accounts are referred to by this. */
  id: string;
  label: string;
  issuer: string;
  name: string;
  vendor: VendorId;
  code: string;
  seconds_left: number;
  /** Length of one code window, for drawing how much of it is left. */
  period_seconds: number;
}

export const getVaultStatus = () => invoke<VaultStatus>("vault_status");
export const createVault = (passphrase: string) => invoke<void>("create_vault", { passphrase });
export const unlockVault = (passphrase: string) => invoke<void>("unlock", { passphrase });
export const lockVault = () => invoke<void>("lock");
export const listCodes = () => invoke<CodeView[]>("list_codes");
export interface AccountDetails {
  issuer: string;
  name: string;
  vendor: VendorId;
}

export const copyCode = (accountId: string) => invoke<void>("copy_code", { accountId });
export const renameAccount = (accountId: string, details: AccountDetails) =>
  invoke<void>("rename_account", { accountId, details });
export const deleteAccount = (accountId: string) => invoke<void>("delete_account", { accountId });

export interface ImportSummary {
  added: number;
  skipped_duplicates: number;
  /** Entries left out because they are unsupported (HOTP, Steam) or unreadable. */
  skipped_unsupported: number;
}

/**
 * Imports a link or the contents of an export file. `vendor` tags accounts whose
 * format does not say where they came from; omit it for "other".
 */
export const importText = (text: string, vendor?: VendorId) =>
  invoke<ImportSummary>("import_text", { text, vendor: vendor ?? null });

export interface Settings {
  /** `null` means never lock automatically. */
  auto_lock_minutes: number | null;
  /** Whether the window asks the system to stay out of screenshots and recordings. */
  capture_protection: boolean;
  /** Whether codes show as dots until revealed. */
  hide_codes: boolean;
  /** Whether the tray menu lists codes; off means labels only. */
  tray_shows_codes: boolean;
  /** Whether the vault locks when the computer sleeps or the screen locks. */
  lock_on_sleep: boolean;
  /** How long a copied code stays on the clipboard. */
  clipboard_clear_seconds: number;
  /** False where the system has no way to block screen capture (Linux). */
  capture_protection_supported: boolean;
}

export const getSettings = () => invoke<Settings>("get_settings");
export const setAutoLockMinutes = (minutes: number | null) =>
  invoke<Settings>("set_auto_lock_minutes", { minutes });
export const setCaptureProtection = (enabled: boolean) => invoke<Settings>("set_capture_protection", { enabled });
export const setHideCodes = (hidden: boolean) => invoke<Settings>("set_hide_codes", { hidden });
export const setTrayShowsCodes = (shown: boolean) => invoke<Settings>("set_tray_shows_codes", { shown });
export const setLockOnSleep = (enabled: boolean) => invoke<Settings>("set_lock_on_sleep", { enabled });
export const setClipboardClearSeconds = (seconds: number) => invoke<Settings>("set_clipboard_clear_seconds", { seconds });
export const recordActivity = () => invoke<void>("record_activity");

export type RelocationOutcome = "unchanged" | "moved" | "used_existing" | "selected";

export const getVaultFolder = () => invoke<string>("get_vault_folder");
/** `null` means the user cancelled the folder picker. */
export const chooseVaultFolder = () => invoke<RelocationOutcome | null>("choose_vault_folder");
