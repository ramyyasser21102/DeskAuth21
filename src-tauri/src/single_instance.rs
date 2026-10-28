//! Only one copy of the app runs per person: starting it again brings the running one forward.

use crate::data_dir_override::{is_overridden, DATA_DIR_VARIABLE};
use crate::window::show_main_window;
use tauri::plugin::TauriPlugin;
use tauri::Wry;

const UNGUARDED_PLUGIN_NAME: &str = "single-instance-off";

/// A copy with its own data folder is deliberately a second copy, which is how sync is tried
/// on one computer, so it is not held to the guard and gets a plugin that does nothing.
pub fn plugin() -> TauriPlugin<Wry> {
    if is_overridden(std::env::var_os(DATA_DIR_VARIABLE).as_ref()) {
        return tauri::plugin::Builder::new(UNGUARDED_PLUGIN_NAME).build();
    }
    tauri_plugin_single_instance::init(|app, _arguments, _working_directory| show_main_window(app))
}
