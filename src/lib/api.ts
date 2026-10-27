// The only module that talks to the Rust backend. Components call these typed
// functions and never `invoke` directly.
import { invoke } from "@tauri-apps/api/core";

export type VaultStatus = "uninitialized" | "locked" | "unlocked";

export interface CodeView {
  /** Stable across devices and reordering; accounts are referred to by this. */
  id: string;
  label: string;
  issuer: string;
  name: string;
  code: string;
  seconds_left: number;
}

export const getVaultStatus = () => invoke<VaultStatus>("vault_status");
export const createVault = (passphrase: string) => invoke<void>("create_vault", { passphrase });
export const unlockVault = (passphrase: string) => invoke<void>("unlock", { passphrase });
export const lockVault = () => invoke<void>("lock");
export const listCodes = () => invoke<CodeView[]>("list_codes");
export interface AccountLabels {
  issuer: string;
  name: string;
}

export const copyCode = (accountId: string) => invoke<void>("copy_code", { accountId });
export const renameAccount = (accountId: string, labels: AccountLabels) =>
  invoke<void>("rename_account", { accountId, labels });
export const deleteAccount = (accountId: string) => invoke<void>("delete_account", { accountId });

export interface ImportSummary {
  added: number;
  skipped_duplicates: number;
}

export const importUri = (uri: string) => invoke<ImportSummary>("import_uri", { uri });

export interface Settings {
  /** `null` means never lock automatically. */
  auto_lock_minutes: number | null;
}

export const getSettings = () => invoke<Settings>("get_settings");
export const setAutoLockMinutes = (minutes: number | null) =>
  invoke<Settings>("set_auto_lock_minutes", { minutes });
export const recordActivity = () => invoke<void>("record_activity");

export type RelocationOutcome = "unchanged" | "moved" | "used_existing" | "selected";

export const getVaultFolder = () => invoke<string>("get_vault_folder");
/** `null` means the user cancelled the folder picker. */
export const chooseVaultFolder = () => invoke<RelocationOutcome | null>("choose_vault_folder");
