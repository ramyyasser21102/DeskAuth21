//! Launch at login. The OS setting is read once at startup and cached in
//! `AppState`, because the tray menu is rebuilt every second.

use crate::error::CommandError;
use crate::launch_options::MINIMIZED_FLAG;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![MINIMIZED_FLAG]))
}

pub fn load_initial_state(app: &AppHandle) {
    // If the OS cannot say, show it as off; toggling will then try to enable it.
    let is_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    app.state::<AppState>().autostart_enabled.store(is_enabled, Ordering::Relaxed);
}

pub fn toggle(app: &AppHandle) -> Result<(), CommandError> {
    let state = app.state::<AppState>();
    let manager = app.autolaunch();
    let was_enabled = state.autostart_enabled.load(Ordering::Relaxed);
    let result = if was_enabled { manager.disable() } else { manager.enable() };
    result.map_err(|error| CommandError::Autostart(error.to_string()))?;
    state.autostart_enabled.store(!was_enabled, Ordering::Relaxed);
    Ok(())
}
