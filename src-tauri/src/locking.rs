//! Locking the vault, and clearing everything else that could still show or hold a code.

use crate::clipboard::clear_copied_code;
use crate::state::AppState;
use crate::tray::refresh_tray;
use crate::vault_actions::lock_vault;
use tauri::{AppHandle, Manager};

pub fn lock_and_clean_up(app: &AppHandle) {
    lock_vault(&app.state::<AppState>());
    clean_up_after_lock(app);
}

/// For callers that locked the vault themselves: the clipboard and the tray menu
/// must not keep what the lock was meant to remove.
pub fn clean_up_after_lock(app: &AppHandle) {
    clear_copied_code(app);
    refresh_tray(app);
}
