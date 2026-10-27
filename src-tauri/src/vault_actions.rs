//! What the user can do with the vault. No Tauri types, so it is unit-testable.

use crate::clock::current_unix_seconds;
use crate::codes::{build_code_views, CodeView};
use crate::error::CommandError;
use crate::import::import_uri;
use crate::passphrase::validate_new_passphrase;
use crate::persistence::{append_and_save, read_vault_file, save_vault};
use crate::state::AppState;
use crate::vault::{KdfParams, Session};

pub fn create_new_vault(state: &AppState, passphrase: &str, kdf_params: KdfParams) -> Result<(), CommandError> {
    validate_new_passphrase(passphrase)?;
    if state.vault_path.exists() {
        return Err(CommandError::VaultAlreadyExists);
    }
    let session = Session::create(passphrase, kdf_params)?;
    let mut vault = state.vault_guard();
    vault.unlock(session, Vec::new());
    let saved = save_vault(&state.vault_path, &vault);
    if saved.is_err() {
        vault.relock();
    }
    saved
}

pub fn unlock_vault(state: &AppState, passphrase: &str) -> Result<(), CommandError> {
    let file_bytes = read_vault_file(&state.vault_path)?;
    let (session, accounts) = Session::open(&file_bytes, passphrase)?;
    state.vault_guard().unlock(session, accounts);
    Ok(())
}

pub fn lock_vault(state: &AppState) {
    state.vault_guard().relock();
}

/// Returns how many accounts were imported.
pub fn import_accounts(state: &AppState, uri: &str) -> Result<usize, CommandError> {
    let imported = import_uri(uri)?;
    let imported_count = imported.len();
    append_and_save(&state.vault_path, &mut state.vault_guard(), imported)?;
    Ok(imported_count)
}

pub fn current_codes(state: &AppState, unix_seconds: u64) -> Result<Vec<CodeView>, CommandError> {
    let vault = state.vault_guard();
    vault.session()?;
    Ok(build_code_views(vault.accounts(), unix_seconds))
}

/// The code for one account, by its position in the list shown to the user.
pub fn code_for_account(state: &AppState, account_index: usize, unix_seconds: u64) -> Result<String, CommandError> {
    current_codes(state, unix_seconds)?
        .into_iter()
        .nth(account_index)
        .map(|view| view.code)
        .ok_or(CommandError::AccountNotFound(account_index))
}

pub fn current_codes_now(state: &AppState) -> Result<Vec<CodeView>, CommandError> {
    current_codes(state, current_unix_seconds())
}

#[cfg(test)]
mod tests;
