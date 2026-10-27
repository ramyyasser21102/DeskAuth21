//! DeskAuth21: wires the modules together. No business logic lives here.

mod autostart;
mod clipboard;
mod clock;
mod codes;
mod commands;
mod dedupe;
mod copy_policy;
mod error;
mod hotkey;
mod import;
mod launch_options;
mod otp;
mod passphrase;
mod persistence;
mod state;
mod tray;
mod vault;
mod vault_actions;
mod window;
mod window_visibility;

use launch_options::should_start_hidden;
use state::AppState;
use tauri::Manager;

const VAULT_FILE_NAME: &str = "vault.da21";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(autostart::plugin())
        .plugin(hotkey::plugin())
        .on_window_event(window::keep_running_in_tray)
        .invoke_handler(tauri::generate_handler![
            commands::vault_status,
            commands::create_vault,
            commands::unlock,
            commands::lock,
            commands::import_uri,
            commands::list_codes,
            commands::copy_code,
        ])
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let vault_path = app.path().app_data_dir()?.join(VAULT_FILE_NAME);
    app.manage(AppState::new(vault_path));
    autostart::load_initial_state(app.handle());
    hotkey::register(app.handle());
    tray::create_tray(app.handle())?;
    tray::spawn_refresh_loop(app.handle().clone());
    if !should_start_hidden(std::env::args()) {
        window::show_main_window(app.handle());
    }
    Ok(())
}
