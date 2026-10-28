//! Launch at login, for the Settings page. The tray menu has its own entry in `tray`.

use crate::autostart;
use crate::error::CommandError;
use crate::tray::refresh_tray;
use tauri::AppHandle;

#[tauri::command]
pub fn get_launch_at_login(app: AppHandle) -> bool {
    autostart::is_enabled(&app)
}

#[tauri::command]
pub fn set_launch_at_login(enabled: bool, app: AppHandle) -> Result<bool, CommandError> {
    let result = autostart::set_enabled(&app, enabled);
    refresh_tray(&app);
    result
}
