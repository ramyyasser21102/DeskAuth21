//! The global show/hide hotkey.

use crate::state::AppState;
use crate::window::toggle_main_window;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Cmd on macOS, Ctrl elsewhere. "A" for Authenticator.
pub const TOGGLE_WINDOW_HOTKEY: &str = "CmdOrCtrl+Shift+A";

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                app.state::<AppState>().record_activity();
                toggle_main_window(app);
            }
        })
        .build()
}

/// Another app may already own the combination; that must not stop DeskAuth21 starting.
pub fn register(app: &AppHandle) {
    if let Err(error) = app.global_shortcut().register(TOGGLE_WINDOW_HOTKEY) {
        eprintln!("could not register the {TOGGLE_WINDOW_HOTKEY} hotkey: {error}");
    }
}
