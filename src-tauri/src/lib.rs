//! DeskAuth21: wires the modules together. No business logic lives here.

mod account_changes;
mod activity;
mod auto_lock;
mod auto_lock_policy;
mod autostart;
mod clipboard;
mod clock;
mod conflict_copies;
mod codes;
mod commands;
mod disk_sync;
mod disk_watch;
mod copy_policy;
mod error;
mod hotkey;
mod import;
mod launch_options;
mod otp;
mod passphrase;
mod record;
mod merge;
mod persistence;
mod settings;
mod settings_store;
mod state;
#[cfg(test)]
mod test_support;
mod tray;
mod vault;
mod vault_actions;
mod vault_location;
mod vault_relocation;
mod window;
mod window_visibility;

use launch_options::should_start_hidden;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
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
            commands::rename_account,
            commands::delete_account,
            commands::get_settings,
            commands::set_auto_lock_minutes,
            commands::record_activity,
            commands::get_vault_folder,
            commands::choose_vault_folder,
        ])
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new(app.path().app_data_dir()?);
    state.replace_settings(settings_store::load_settings(&state.settings_path()));
    app.manage(state);
    autostart::load_initial_state(app.handle());
    hotkey::register(app.handle());
    tray::create_tray(app.handle())?;
    tray::spawn_refresh_loop(app.handle().clone());
    auto_lock::spawn_loop(app.handle().clone());
    disk_watch::spawn_loop(app.handle().clone());
    if !should_start_hidden(std::env::args()) {
        window::show_main_window(app.handle());
    }
    Ok(())
}
