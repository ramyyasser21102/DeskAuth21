// The only module that talks to the Rust backend. Components call these typed
// functions and never `invoke` directly.
import { invoke } from "@tauri-apps/api/core";

export type VaultStatus = "uninitialized" | "locked" | "unlocked";

export interface CodeView {
  label: string;
  code: string;
  seconds_left: number;
}

export const getVaultStatus = () => invoke<VaultStatus>("vault_status");
export const createVault = (passphrase: string) => invoke<void>("create_vault", { passphrase });
export const unlockVault = (passphrase: string) => invoke<void>("unlock", { passphrase });
export const lockVault = () => invoke<void>("lock");
export const listCodes = () => invoke<CodeView[]>("list_codes");
export const copyCode = (accountIndex: number) => invoke<void>("copy_code", { accountIndex });

export interface ImportSummary {
  added: number;
  skipped_duplicates: number;
}

export const importUri = (uri: string) => invoke<ImportSummary>("import_uri", { uri });
