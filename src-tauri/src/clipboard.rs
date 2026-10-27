//! Copies one-time codes to the system clipboard and clears them again.

use crate::clock::current_unix_seconds;
use crate::copy_policy::{should_clear_clipboard, CLIPBOARD_CLEAR_DELAY};
use crate::error::CommandError;
use crate::state::AppState;
use crate::vault_actions::code_for_account;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Counts copies so only the newest one's timer may clear the clipboard.
static LATEST_COPY_NUMBER: AtomicU64 = AtomicU64::new(0);

pub fn copy_code_for_account(app: &AppHandle, account_id: &str) -> Result<(), CommandError> {
    let code = code_for_account(&app.state::<AppState>(), account_id, current_unix_seconds())?;
    write_text(app, code.clone())?;
    app.state::<AppState>().record_activity();
    let copy_number = LATEST_COPY_NUMBER.fetch_add(1, Ordering::SeqCst) + 1;
    schedule_clear(app.clone(), code, copy_number);
    Ok(())
}

fn write_text(app: &AppHandle, text: String) -> Result<(), CommandError> {
    app.clipboard()
        .write_text(text)
        .map_err(|error| CommandError::Clipboard(error.to_string()))
}

fn schedule_clear(app: AppHandle, copied_code: String, copy_number: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(CLIPBOARD_CLEAR_DELAY);
        clear_if_unchanged(&app, &copied_code, copy_number);
    });
}

fn clear_if_unchanged(app: &AppHandle, copied_code: &str, copy_number: u64) {
    let is_latest_copy = LATEST_COPY_NUMBER.load(Ordering::SeqCst) == copy_number;
    let current_contents = app.clipboard().read_text().ok();
    if !should_clear_clipboard(current_contents.as_deref(), copied_code, is_latest_copy) {
        return;
    }
    // Never log the code itself, only that clearing failed.
    if let Err(error) = write_text(app, String::new()) {
        eprintln!("could not clear the clipboard: {error}");
    }
}
