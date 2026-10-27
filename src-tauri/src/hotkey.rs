//! Registers the global keyboard shortcuts with the system.

use crate::locking::lock_and_clean_up;
use crate::shortcuts::{ShortcutAction, ShortcutSettings};
use crate::state::AppState;
use crate::window::toggle_main_window;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn run_action(app: &AppHandle, action: ShortcutAction) {
    app.state::<AppState>().record_activity();
    match action {
        ShortcutAction::ToggleWindow => toggle_main_window(app),
        ShortcutAction::LockVault => lock_and_clean_up(app),
    }
}

fn run_when_pressed(app: &AppHandle, action: ShortcutAction, state: ShortcutState) {
    if state == ShortcutState::Pressed {
        run_action(app, action);
    }
}

/// Replaces whatever is registered with exactly `shortcuts`. Stops at the first one
/// the system refuses, because another app may already own the combination.
pub fn register_all(app: &AppHandle, shortcuts: &ShortcutSettings) -> Result<(), tauri_plugin_global_shortcut::Error> {
    let registry = app.global_shortcut();
    registry.unregister_all()?;
    for (action, text) in shortcuts.assigned() {
        registry.on_shortcut(text, move |app, _shortcut, event| run_when_pressed(app, action, event.state()))?;
    }
    Ok(())
}

/// A refusal must not stop DeskAuth21 starting; the person can pick another in Settings.
pub fn apply(app: &AppHandle) {
    let shortcuts = app.state::<AppState>().settings().shortcuts;
    if let Err(error) = register_all(app, &shortcuts) {
        eprintln!("could not register a keyboard shortcut: {error}");
    }
}
