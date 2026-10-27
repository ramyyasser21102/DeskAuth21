//! What the user can do with the vault. No Tauri types, so it is unit-testable.

use crate::clock::current_unix_seconds;
use crate::auto_lock_policy::{is_allowed_minutes, should_auto_lock};
use crate::codes::{build_code_views, CodeView};
use crate::dedupe::split_new_accounts;
use crate::error::CommandError;
use crate::import::import_uri;
use crate::passphrase::validate_new_passphrase;
use crate::disk_sync::{sync_from_disk, DiskSyncOutcome};
use crate::persistence::{append_and_save, fingerprint, read_vault_file, save_vault};
use crate::settings::Settings;
use crate::settings_store::save_settings;
use crate::state::AppState;
use crate::vault::{KdfParams, Session};
use std::time::Instant;

pub fn create_new_vault(state: &AppState, passphrase: &str, kdf_params: KdfParams) -> Result<(), CommandError> {
    validate_new_passphrase(passphrase)?;
    let vault_path = state.vault_path();
    if vault_path.exists() {
        return Err(CommandError::VaultAlreadyExists);
    }
    let session = Session::create(passphrase, kdf_params)?;
    let mut vault = state.vault_guard();
    vault.unlock(session, Vec::new());
    let saved = save_vault(&vault_path, &mut vault);
    if saved.is_err() {
        vault.relock();
    }
    state.record_activity();
    saved
}

pub fn unlock_vault(state: &AppState, passphrase: &str) -> Result<(), CommandError> {
    let file_bytes = read_vault_file(&state.vault_path())?;
    let (session, accounts) = Session::open(&file_bytes, passphrase)?;
    let mut vault = state.vault_guard();
    vault.unlock(session, accounts);
    vault.set_disk_fingerprint(Some(fingerprint(&file_bytes)));
    state.record_activity();
    Ok(())
}

pub fn lock_vault(state: &AppState) {
    state.vault_guard().relock();
}

/// What an import did, so the UI can tell the user.
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct ImportSummary {
    pub added: usize,
    pub skipped_duplicates: usize,
}

/// Imports accounts from a URI, skipping any whose secret the vault already holds.
pub fn import_accounts(state: &AppState, uri: &str) -> Result<ImportSummary, CommandError> {
    let imported = import_uri(uri)?;
    let mut vault = state.vault_guard();
    vault.session()?;
    // Pick up anything another device added first, so we neither duplicate it
    // nor overwrite it. If the file is a different vault entirely, stop.
    if sync_from_disk(&state.vault_path(), &mut vault) == DiskSyncOutcome::Unreadable {
        return Err(CommandError::VaultChangedElsewhere);
    }
    let (new_accounts, skipped_duplicates) = split_new_accounts(vault.accounts(), imported);
    let added = new_accounts.len();
    if added > 0 {
        append_and_save(&state.vault_path(), &mut vault, new_accounts)?;
    }
    state.record_activity();
    Ok(ImportSummary { added, skipped_duplicates })
}

/// Locks the vault if it is unlocked and has been idle past the user's timeout.
/// Returns whether it locked. `now` is passed in so tests control the clock.
pub fn lock_if_idle(state: &AppState, now: Instant) -> bool {
    let mut vault = state.vault_guard();
    let idle_for = state.activity.idle_for(now);
    if !vault.is_unlocked() || !should_auto_lock(idle_for, state.settings().auto_lock_timeout()) {
        return false;
    }
    vault.relock();
    true
}

/// Validates, saves and applies a new auto-lock timeout (`None` = never).
pub fn update_auto_lock(state: &AppState, minutes: Option<u32>) -> Result<Settings, CommandError> {
    if !is_allowed_minutes(minutes) {
        return Err(CommandError::InvalidSetting(format!("auto-lock of {minutes:?} minutes")));
    }
    let mut settings = state.settings();
    settings.auto_lock_minutes = minutes;
    save_settings(&state.settings_path(), &settings)?;
    state.replace_settings(settings.clone());
    state.record_activity();
    Ok(settings)
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
