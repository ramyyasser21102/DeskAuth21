//! Copies one-time codes to the system clipboard and takes them off again: when a
//! timer runs out, when the vault locks, and when the app quits.

use crate::clock::current_unix_seconds;
use crate::copy_policy::{clear_delay, should_clear_clipboard};
use crate::copy_registry::CopyRegistry;
use crate::error::CommandError;
use crate::state::AppState;
use crate::vault_actions::code_for_account;
use std::sync::{Mutex, PoisonError};
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

static COPIES: Mutex<CopyRegistry> = Mutex::new(CopyRegistry::new());

pub fn copy_code_for_account(app: &AppHandle, account_id: &str) -> Result<(), CommandError> {
    let state = app.state::<AppState>();
    let code = code_for_account(&state, account_id, current_unix_seconds())?;
    write_text(app, code.clone())?;
    let copy_number = COPIES.lock().unwrap_or_else(PoisonError::into_inner).record(code);
    state.record_activity();
    schedule_clear(app.clone(), copy_number, clear_delay(state.settings().clipboard_clear_seconds));
    Ok(())
}

/// Takes the last copied code off the clipboard now, if it is still there. Called when
/// the vault locks and when the app quits.
pub fn clear_copied_code(app: &AppHandle) {
    clear_copy(app, None);
}

fn write_text(app: &AppHandle, text: String) -> Result<(), CommandError> {
    app.clipboard()
        .write_text(text)
        .map_err(|error| CommandError::Clipboard(error.to_string()))
}

fn schedule_clear(app: AppHandle, copy_number: u64, delay: std::time::Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        clear_copy(&app, Some(copy_number));
    });
}

/// With a copy number, only that copy is cleared and only if it is still the newest.
fn clear_copy(app: &AppHandle, only_copy_number: Option<u64>) {
    let copied = COPIES.lock().unwrap_or_else(PoisonError::into_inner).take(only_copy_number);
    let Some(copied) = copied else {
        return;
    };
    let current_contents = app.clipboard().read_text().ok();
    if !should_clear_clipboard(current_contents.as_deref(), &copied.code, true) {
        return;
    }
    // Never log the code itself, only that clearing failed.
    if let Err(error) = write_text(app, String::new()) {
        eprintln!("could not clear the clipboard: {error}");
    }
}
