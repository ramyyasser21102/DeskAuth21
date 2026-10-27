//! How the vault is protected: its file format, its passphrase and its recovery key.
//! No Tauri types, so it is unit-testable.

use crate::error::CommandError;
use crate::passphrase::validate_new_passphrase;
use crate::persistence::{fingerprint, read_vault_file, save_vault, write_backup_copy};
use crate::recovery_key;
use crate::state::{AppState, VaultState};
use crate::vault::{KdfParams, Session};
use crate::vault_actions::with_synced_vault;
use std::path::Path;

/// Ends in neither `.da21` nor anything the sync code looks at, so it is never merged.
const V1_BACKUP_SUFFIX: &str = ".v1.bak";

/// Puts `updated` in place of the open session and saves. If saving fails the previous
/// session is put back, so memory never claims a protection the file does not have.
fn swap_session_and_save(vault: &mut VaultState, path: &Path, updated: Session) -> Result<(), CommandError> {
    let previous = vault.replace_session(updated);
    let saved = save_vault(path, vault);
    if let (Err(_), Some(previous)) = (&saved, previous) {
        vault.replace_session(previous);
    }
    saved
}

/// Which file format the unlocked vault is in: 1 for vaults made before unlock slots.
pub fn vault_format_version(state: &AppState) -> Result<u8, CommandError> {
    Ok(state.vault_guard().session()?.format_version())
}

/// Moves an older vault to the current format. The old file is kept beside it first,
/// and if saving fails the vault stays exactly as it was.
pub fn upgrade_vault_format(state: &AppState) -> Result<(), CommandError> {
    with_synced_vault(state, |vault, path| {
        let Some(upgraded) = vault.session()?.upgraded()? else {
            return Ok(());
        };
        write_backup_copy(path, V1_BACKUP_SUFFIX)?;
        swap_session_and_save(vault, path, upgraded)
    })
}

/// Needs the current passphrase even though the vault is open, so someone at an
/// unlocked computer cannot lock the owner out.
pub fn change_passphrase(state: &AppState, current: &str, new_passphrase: &str, params: KdfParams) -> Result<(), CommandError> {
    validate_new_passphrase(new_passphrase)?;
    with_synced_vault(state, |vault, path| {
        let mut updated = vault.session()?.clone();
        updated.change_passphrase(current, new_passphrase, params)?;
        swap_session_and_save(vault, path, updated)
    })
}

pub fn has_recovery_key(state: &AppState) -> Result<bool, CommandError> {
    Ok(state.vault_guard().session()?.has_recovery_key())
}

/// Makes a recovery key without saving it: nothing changes until the person has proved
/// they wrote it down and `save_recovery_key` is called with it.
pub fn generate_recovery_key(state: &AppState) -> Result<String, CommandError> {
    state.vault_guard().session()?;
    recovery_key::generate()
        .map(|phrase| phrase.to_string())
        .map_err(|_| crate::vault::VaultError::Random.into())
}

/// Makes `phrase` the vault's recovery key, replacing any earlier one. Needs the current passphrase.
pub fn save_recovery_key(state: &AppState, phrase: &str, current_passphrase: &str) -> Result<(), CommandError> {
    let normalized = recovery_key::normalize(phrase)?;
    with_synced_vault(state, |vault, path| {
        let mut updated = vault.session()?.clone();
        updated.set_recovery_key(&normalized, current_passphrase)?;
        swap_session_and_save(vault, path, updated)
    })
}

/// For someone who forgot the passphrase: the recovery key opens the vault, a new
/// passphrase replaces the old one, and the vault stays unlocked.
pub fn reset_passphrase_with_recovery_key(state: &AppState, phrase: &str, new_passphrase: &str, params: KdfParams) -> Result<(), CommandError> {
    validate_new_passphrase(new_passphrase)?;
    let normalized = recovery_key::normalize(phrase)?;
    let path = state.vault_path();
    let file_bytes = read_vault_file(&path)?;
    let (mut session, data) = Session::open_with_recovery(&file_bytes, &normalized)?;
    session.set_passphrase(new_passphrase, params)?;
    let mut vault = state.vault_guard();
    vault.unlock(session, data);
    vault.set_disk_fingerprint(Some(fingerprint(&file_bytes)));
    let saved = save_vault(&path, &mut vault);
    if saved.is_err() {
        vault.relock();
    }
    state.record_activity();
    saved
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod passphrase_tests;
#[cfg(test)]
mod recovery_tests;
