//! Commands for requiring a hardware key. Thin adapters over `vault_security`; the slow ones
//! are async so waiting for a touch never blocks the window.

use crate::error::CommandError;
use crate::key_device::KeyRegistration;
use crate::state::AppState;
use crate::usb_key;
use crate::vault_security::{self, to_hex};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct HardwareKeyView {
    /// Hex of the credential id; the window hands it back to remove a key.
    pub id: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct HardwareKeyStatus {
    /// False where this system cannot talk to security keys (Windows, for now).
    pub is_supported: bool,
    /// Whether unlocking needs a key as well as the passphrase.
    pub is_required: bool,
    pub keys: Vec<HardwareKeyView>,
}

/// Whether the unlock screen must ask for a hardware key; works while the vault is locked.
#[tauri::command]
pub fn unlock_needs_hardware_key(state: State<AppState>) -> Result<bool, CommandError> {
    vault_security::unlock_needs_hardware_key(&state)
}

#[tauri::command]
pub fn hardware_key_status(state: State<AppState>) -> Result<HardwareKeyStatus, CommandError> {
    let (is_required, keys) = vault_security::hardware_keys(&state)?;
    let keys = keys.into_iter().map(|key| HardwareKeyView { id: to_hex(&key.credential_id), label: key.label }).collect();
    Ok(HardwareKeyStatus { is_supported: usb_key::is_supported(), is_required, keys })
}

#[tauri::command]
pub async fn enable_hardware_key(current_passphrase: String, pin: Option<String>, label: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    let device = usb_key::platform_device();
    let key = KeyRegistration { device: device.as_ref(), pin: pin.as_deref(), label: &label };
    vault_security::enable_hardware_key(&state, &key, &current_passphrase)
}

#[tauri::command]
pub async fn add_hardware_key(pin: Option<String>, label: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    let device = usb_key::platform_device();
    let key = KeyRegistration { device: device.as_ref(), pin: pin.as_deref(), label: &label };
    vault_security::add_hardware_key(&state, &key)
}

#[tauri::command]
pub fn remove_hardware_key(id: String, state: State<AppState>) -> Result<(), CommandError> {
    vault_security::remove_hardware_key(&state, &id)
}

#[tauri::command]
pub async fn disable_hardware_keys(current_passphrase: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    vault_security::disable_hardware_keys(&state, &current_passphrase)
}
