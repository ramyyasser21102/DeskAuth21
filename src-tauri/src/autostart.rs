//! Launch at login. The system's own setting is the truth; `AppState` keeps a copy only so
//! the tray menu, which is rebuilt every second, never has to ask the system.

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

fn remember(app: &AppHandle, is_enabled: bool) {
    app.state::<AppState>().autostart_enabled.store(is_enabled, Ordering::Relaxed);
}

fn failure(error: impl std::fmt::Display) -> CommandError {
    CommandError::Autostart(error.to_string())
}

pub fn load_initial_state(app: &AppHandle) {
    // If the system cannot say, show it as off; turning it on will then try to enable it.
    remember(app, app.autolaunch().is_enabled().unwrap_or(false));
}

/// What the system says right now, which can differ from what this app last saw
/// (the person may have removed the login item in System Settings).
pub fn is_enabled(app: &AppHandle) -> bool {
    let is_on = app.autolaunch().is_enabled().unwrap_or_else(|_| app.state::<AppState>().autostart_enabled.load(Ordering::Relaxed));
    remember(app, is_on);
    is_on
}

/// Turns launch at login on or off and checks the system kept the change, because
/// a silent failure would leave the person believing it worked.
pub fn set_enabled(app: &AppHandle, wanted: bool) -> Result<bool, CommandError> {
    let manager = app.autolaunch();
    let changed = if wanted { manager.enable() } else { manager.disable() };
    changed.map_err(failure)?;
    let actual = manager.is_enabled().map_err(failure)?;
    remember(app, actual);
    if actual == wanted {
        return Ok(actual);
    }
    Err(failure("the system did not keep the change"))
}

pub fn toggle(app: &AppHandle) -> Result<bool, CommandError> {
    set_enabled(app, !is_enabled(app))
}
