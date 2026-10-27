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

export interface PassphraseStrength {
  /** 0 (very weak) to 4 (strong). */
  score: number;
  feedback: string | null;
}

/** Advice on a new passphrase while it is typed. The text stays inside the app. */
export const assessPassphrase = (passphrase: string) => invoke<PassphraseStrength>("assess_passphrase", { passphrase });
/** Six random words, made by the app. */
export const generatePassphrase = () => invoke<string>("generate_passphrase");

/** The page of the main window to bring forward from the quick panel. */
export type AppPage = "codes" | "import" | "settings";
export const hideQuickPanel = () => invoke<void>("hide_quick_panel");
export const openAppPage = (page: AppPage) => invoke<void>("open_app_page", { page });

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
/** `pin` is the security key's own PIN, only needed if the key has one. */
export const unlockVault = (passphrase: string, pin?: string) => invoke<void>("unlock", { passphrase, pin: pin ?? null });
/** Whether unlocking will ask for a hardware key; readable while the vault is locked. */
export const unlockNeedsHardwareKey = () => invoke<boolean>("unlock_needs_hardware_key");

export interface HardwareKeyEntry {
  /** Hand this back to remove the key. */
  id: string;
  label: string;
}

export interface HardwareKeyStatus {
  /** False where this system cannot talk to security keys. */
  is_supported: boolean;
  /** Whether unlocking needs a key as well as the passphrase. */
  is_required: boolean;
  keys: HardwareKeyEntry[];
}

export const getHardwareKeyStatus = () => invoke<HardwareKeyStatus>("hardware_key_status");
export const enableHardwareKey = (currentPassphrase: string, pin: string | null, label: string) =>
  invoke<void>("enable_hardware_key", { currentPassphrase, pin, label });
export const addHardwareKey = (pin: string | null, label: string) => invoke<void>("add_hardware_key", { pin, label });
export const removeHardwareKey = (id: string) => invoke<void>("remove_hardware_key", { id });
export const disableHardwareKeys = (currentPassphrase: string) => invoke<void>("disable_hardware_keys", { currentPassphrase });
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

/** 1 for a vault made before unlock slots, 2 for the current format. */
export const getVaultFormatVersion = () => invoke<number>("vault_format_version");
export const upgradeVaultFormat = () => invoke<void>("upgrade_vault_format");

export const changePassphrase = (current: string, newPassphrase: string) =>
  invoke<void>("change_passphrase", { current, newPassphrase });
export const hasRecoveryKey = () => invoke<boolean>("has_recovery_key");
/** Makes the 24 words without saving them; nothing changes until `saveRecoveryKey`. */
export const generateRecoveryKey = () => invoke<string>("generate_recovery_key");
export const saveRecoveryKey = (phrase: string, currentPassphrase: string) =>
  invoke<void>("save_recovery_key", { phrase, currentPassphrase });
/** Opens the vault with the recovery key and sets a new passphrase. */
export const resetPassphraseWithRecoveryKey = (phrase: string, newPassphrase: string) =>
  invoke<void>("reset_passphrase_with_recovery_key", { phrase, newPassphrase });

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
