//! Showing, hiding and toggling the main window. The app lives in the tray, so
//! closing the window hides it instead of quitting.

use crate::quick_panel;
use crate::window_visibility::{action_for_toggle, VisibilityAction};
use tauri::{AppHandle, Manager, WebviewWindow, Window, WindowEvent};

const MAIN_WINDOW_LABEL: &str = "main";

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
}

fn log_if_failed(action: &str, result: tauri::Result<()>) {
    if let Err(error) = result {
        eprintln!("could not {action} the window: {error}");
    }
}

pub fn show_main_window(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    log_if_failed("show", window.show());
    log_if_failed("focus", window.set_focus());
}

pub fn hide_main_window(app: &AppHandle) {
    if let Some(window) = main_window(app) {
        log_if_failed("hide", window.hide());
    }
}

pub fn toggle_main_window(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    // If the OS cannot say, assume hidden so the hotkey errs towards showing the window.
    let is_visible = window.is_visible().unwrap_or(false);
    let is_focused = window.is_focused().unwrap_or(false);
    match action_for_toggle(is_visible, is_focused) {
        VisibilityAction::Show => show_main_window(app),
        VisibilityAction::Hide => hide_main_window(app),
    }
}

/// Window-event hook: turn "close" into "hide" so the tray keeps running.
pub fn keep_running_in_tray(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            log_if_failed("hide", window.hide());
        }
        WindowEvent::Focused(false) => quick_panel::hide_when_focus_is_lost(window),
        _ => {}
    }
}
