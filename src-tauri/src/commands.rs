//! Tauri command handlers: thin adapters from the frontend to `vault_actions`.

use crate::account_changes::AccountDetails;
use crate::clipboard;
use crate::codes::CodeView;
use crate::capture_protection;
use crate::copy_policy::is_allowed_clear_seconds;
use crate::locking::lock_and_clean_up;
use crate::passphrase_generator;
use crate::passphrase_strength::{assess, PassphraseStrength};
use crate::settings::SettingsView;
use crate::error::CommandError;
use crate::state::{AppState, VaultStatus};
use crate::tray::refresh_tray;
use crate::vault::KdfParams;
use crate::vault_actions::{self, ImportSummary};
use crate::usb_key;
use crate::vault_security;
use crate::vault_location::{self, RelocationOutcome};
use crate::vendor::{vendor_catalog, Vendor, VendorInfo};
use tauri::{AppHandle, State, WebviewWindow};
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
pub async fn unlock(passphrase: String, pin: Option<String>, state: State<'_, AppState>, app: AppHandle) -> Result<(), CommandError> {
    vault_actions::unlock_vault_with_key(&state, &passphrase, usb_key::platform_device().as_ref(), pin.as_deref())?;
    refresh_tray(&app);
    Ok(())
}

/// 1 for a vault made before unlock slots, 2 for the current format.
#[tauri::command]
pub fn vault_format_version(state: State<AppState>) -> Result<u8, CommandError> {
    vault_security::vault_format_version(&state)
}

#[tauri::command]
pub fn upgrade_vault_format(state: State<AppState>) -> Result<(), CommandError> {
    vault_security::upgrade_vault_format(&state)
}

// Async so the slow key derivation runs off the main thread.
#[tauri::command]
pub async fn change_passphrase(current: String, new_passphrase: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    vault_security::change_passphrase(&state, &current, &new_passphrase, KdfParams::DEFAULT)
}

#[tauri::command]
pub fn has_recovery_key(state: State<AppState>) -> Result<bool, CommandError> {
    vault_security::has_recovery_key(&state)
}

/// Makes the words without saving them; `save_recovery_key` does that once they are written down.
#[tauri::command]
pub fn generate_recovery_key(state: State<AppState>) -> Result<String, CommandError> {
    vault_security::generate_recovery_key(&state)
}

#[tauri::command]
pub async fn save_recovery_key(phrase: String, current_passphrase: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    vault_security::save_recovery_key(&state, &phrase, &current_passphrase)
}

#[tauri::command]
pub async fn reset_passphrase_with_recovery_key(phrase: String, new_passphrase: String, state: State<'_, AppState>, app: AppHandle) -> Result<(), CommandError> {
    vault_security::reset_passphrase_with_recovery_key(&state, &phrase, &new_passphrase, KdfParams::DEFAULT)?;
    refresh_tray(&app);
    Ok(())
}

#[tauri::command]
pub fn lock(app: AppHandle) {
    lock_and_clean_up(&app);
}

#[tauri::command]
pub fn import_text(text: String, vendor: Option<Vendor>, state: State<AppState>, app: AppHandle) -> Result<ImportSummary, CommandError> {
    let summary = vault_actions::import_accounts(&state, &text, vendor)?;
    refresh_tray(&app);
    Ok(summary)
}

#[tauri::command]
pub fn list_codes(state: State<AppState>) -> Result<Vec<CodeView>, CommandError> {
    vault_actions::current_codes_now(&state)
}

/// Copies the code and clears the clipboard again after a delay.
#[tauri::command]
pub fn copy_code(account_id: String, app: AppHandle) -> Result<(), CommandError> {
    clipboard::copy_code_for_account(&app, &account_id)
}

#[tauri::command]
pub fn rename_account(account_id: String, details: AccountDetails, state: State<AppState>, app: AppHandle) -> Result<(), CommandError> {
    vault_actions::rename_stored_account(&state, &account_id, &details)?;
    refresh_tray(&app);
    Ok(())
}

#[tauri::command]
pub fn delete_account(account_id: String, state: State<AppState>, app: AppHandle) -> Result<(), CommandError> {
    vault_actions::delete_stored_account(&state, &account_id)?;
    refresh_tray(&app);
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> SettingsView {
    state.settings().into()
}

#[tauri::command]
pub fn set_auto_lock_minutes(minutes: Option<u32>, state: State<AppState>) -> Result<SettingsView, CommandError> {
    Ok(vault_actions::update_auto_lock(&state, minutes)?.into())
}

#[tauri::command]
pub fn set_capture_protection(enabled: bool, state: State<AppState>, window: WebviewWindow) -> Result<SettingsView, CommandError> {
    let settings = vault_actions::update_settings(&state, |settings| settings.capture_protection = enabled)?;
    capture_protection::apply(&window, enabled);
    Ok(settings.into())
}

#[tauri::command]
pub fn set_hide_codes(hidden: bool, state: State<AppState>) -> Result<SettingsView, CommandError> {
    Ok(vault_actions::update_settings(&state, |settings| settings.hide_codes = hidden)?.into())
}

/// Advice while the person types a new passphrase. It stays inside this program.
#[tauri::command]
pub fn assess_passphrase(passphrase: String) -> PassphraseStrength {
    assess(&passphrase)
}

#[tauri::command]
pub fn generate_passphrase() -> Result<String, CommandError> {
    passphrase_generator::generate_passphrase().map_err(|_| CommandError::Vault(crate::vault::VaultError::Random))
}

#[tauri::command]
pub fn set_lock_on_sleep(enabled: bool, state: State<AppState>) -> Result<SettingsView, CommandError> {
    Ok(vault_actions::update_settings(&state, |settings| settings.lock_on_sleep = enabled)?.into())
}

#[tauri::command]
pub fn set_clipboard_clear_seconds(seconds: u32, state: State<AppState>) -> Result<SettingsView, CommandError> {
    if !is_allowed_clear_seconds(seconds) {
        return Err(CommandError::InvalidSetting(format!("clearing the clipboard after {seconds} seconds")));
    }
    Ok(vault_actions::update_settings(&state, |settings| settings.clipboard_clear_seconds = seconds)?.into())
}

/// The tray menu is rebuilt by its own refresh loop, so only the setting changes here.
#[tauri::command]
pub fn set_tray_shows_codes(shown: bool, state: State<AppState>) -> Result<SettingsView, CommandError> {
    Ok(vault_actions::update_settings(&state, |settings| settings.tray_shows_codes = shown)?.into())
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

/// The apps accounts can be tagged as coming from, for the pickers in the window.
#[tauri::command]
pub fn list_vendors() -> Vec<VendorInfo> {
    vendor_catalog()
}
