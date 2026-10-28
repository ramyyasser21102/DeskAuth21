//! Saving a copy of the vault, putting one back, and exporting the accounts in the open.
//! The file dialogs live in `commands`; everything here takes the paths it is given.

use crate::error::CommandError;
use crate::otpauth_export::export_otpauth_lines;
use crate::persistence::{read_vault_file, write_atomically, write_backup_copy};
use crate::state::AppState;
use crate::vault::{is_vault_file, VaultError};
use crate::vault_location::copy_vault_file;
use std::path::Path;

/// Left beside the vault when a backup replaces it, so a mistaken restore can be undone by hand.
const BEFORE_RESTORE_SUFFIX: &str = ".before-restore.backup";

/// Copies the encrypted vault file. The copy is as safe as the vault: it needs the same
/// passphrase (and hardware key), so it can be kept anywhere. Works while locked.
pub fn save_backup(state: &AppState, destination: &Path) -> Result<(), CommandError> {
    copy_vault_file(&state.vault_path(), destination)
}

/// Replaces the vault with the backup at `source` and locks, so the next unlock uses the
/// backup's passphrase. The vault it replaces is kept beside it.
pub fn restore_backup(state: &AppState, source: &Path) -> Result<(), CommandError> {
    let bytes = read_vault_file(source)?;
    if !is_vault_file(&bytes) {
        return Err(VaultError::BadFormat.into());
    }
    let vault_path = state.vault_path();
    if vault_path.exists() {
        write_backup_copy(&vault_path, BEFORE_RESTORE_SUFFIX)?;
    }
    let mut vault = state.vault_guard();
    write_atomically(&vault_path, &bytes)?;
    vault.relock();
    Ok(())
}

/// Writes every account as plain `otpauth://` links. Anyone who gets the file can make the
/// codes, so it needs the passphrase again even though the vault is open. Returns how many were written.
pub fn export_accounts(state: &AppState, passphrase: &str, destination: &Path) -> Result<usize, CommandError> {
    let vault = state.vault_guard();
    vault.session()?.verify_passphrase(passphrase)?;
    let records = vault.accounts();
    let text = export_otpauth_lines(records);
    write_atomically(destination, text.as_bytes())?;
    Ok(records.len())
}

#[cfg(test)]
mod tests;
