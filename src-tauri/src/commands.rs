//! Tauri command handlers: thin adapters from the frontend to `vault_actions`.

use crate::clipboard;
use crate::codes::CodeView;
use crate::settings::Settings;
use crate::error::CommandError;
use crate::state::{AppState, VaultStatus};
use crate::tray::refresh_tray;
use crate::vault::KdfParams;
use crate::vault_actions::{self, ImportSummary};
use crate::vault_location::{self, RelocationOutcome};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn vault_status(state: State<AppState>) -> VaultStatus {
    let vault = state.vault_guard();
    state.status_of(&vault)
}

// Async so the deliberately slow key derivation runs off the main thread.
#[tauri::command]
pub async fn create_vault(passphrase: String, state: State<'_, AppState>, app: AppHandle) -> Result<(), CommandError> {
    vault_actions::create_new_vault(&state, &passphrase, KdfParams::DEFAULT)?;
    refresh_tray(&app);
    Ok(())
}

#[tauri::command]
pub async fn unlock(passphrase: String, state: State<'_, AppState>, app: AppHandle) -> Result<(), CommandError> {
    vault_actions::unlock_vault(&state, &passphrase)?;
    refresh_tray(&app);
    Ok(())
}

#[tauri::command]
pub fn lock(state: State<AppState>, app: AppHandle) {
    vault_actions::lock_vault(&state);
    refresh_tray(&app);
}

#[tauri::command]
pub fn import_uri(uri: String, state: State<AppState>, app: AppHandle) -> Result<ImportSummary, CommandError> {
    let summary = vault_actions::import_accounts(&state, &uri)?;
    refresh_tray(&app);
    Ok(summary)
}

#[tauri::command]
pub fn list_codes(state: State<AppState>) -> Result<Vec<CodeView>, CommandError> {
    vault_actions::current_codes_now(&state)
}

/// Copies the code and clears the clipboard again after a delay.
#[tauri::command]
pub fn copy_code(account_index: usize, app: AppHandle) -> Result<(), CommandError> {
    clipboard::copy_code_for_account(&app, account_index)
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn set_auto_lock_minutes(minutes: Option<u32>, state: State<AppState>) -> Result<Settings, CommandError> {
    vault_actions::update_auto_lock(&state, minutes)
}

/// The window reports real user input here; background polling must not.
#[tauri::command]
pub fn record_activity(state: State<AppState>) {
    state.record_activity();
}

/// The folder the vault is in, for display.
#[tauri::command]
pub fn get_vault_folder(state: State<AppState>) -> String {
    let vault_path = state.vault_path();
    vault_path.parent().unwrap_or(&vault_path).display().to_string()
}

/// Asks the user for a folder (a synced one, to share the vault between devices)
/// and switches to it. `None` means they cancelled.
#[tauri::command]
pub async fn choose_vault_folder(state: State<'_, AppState>, app: AppHandle) -> Result<Option<RelocationOutcome>, CommandError> {
    let Some(picked) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };
    let folder = picked.into_path().map_err(|error| CommandError::InvalidSetting(error.to_string()))?;
    let outcome = vault_location::change_vault_folder(&state, &folder)?;
    refresh_tray(&app);
    Ok(Some(outcome))
}
