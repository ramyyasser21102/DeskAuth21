//! DeskAuth21: wires the modules together. No business logic lives here.

mod account_changes;
mod activity;
mod auto_lock;
mod auto_lock_policy;
mod autostart;
mod capture_protection;
mod copy_registry;
mod clipboard;
mod clock;
mod conflict_copies;
mod data_dir_override;
mod codes;
mod commands;
mod commands_hardware;
mod commands_panel;
mod disk_sync;
mod disk_watch;
mod copy_policy;
mod error;
mod hotkey;
mod import;
mod key_device;
mod launch_options;
mod lock_triggers;
mod locking;
mod otp;
mod panel_position;
mod passphrase;
mod passphrase_generator;
mod passphrase_strength;
mod recovery_key;
mod record;
mod screen_lock;
mod secret;
mod merge;
mod persistence;
mod quick_panel;
mod settings;
mod settings_store;
mod state;
mod system_lock_watcher;
#[cfg(test)]
mod test_support;
mod tray;
mod usb_key;
mod tray_model;
mod vault;
mod vendor;
mod vault_actions;
mod vault_security;
mod vault_location;
mod vault_relocation;
mod window;
mod window_visibility;

use launch_options::{should_show_panel, should_start_hidden};
use state::AppState;
use tauri::Manager;

/// Every function the window may call.
fn command_handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // The vault and its protection.
        commands::vault_status, commands::create_vault, commands::unlock, commands::lock,
        commands::vault_format_version, commands::upgrade_vault_format, commands::change_passphrase,
        commands::has_recovery_key, commands::generate_recovery_key, commands::save_recovery_key,
        commands::reset_passphrase_with_recovery_key, commands::assess_passphrase, commands::generate_passphrase,
        commands_panel::hide_quick_panel, commands_panel::open_app_page,
        commands_hardware::unlock_needs_hardware_key, commands_hardware::hardware_key_status, commands_hardware::enable_hardware_key,
        commands_hardware::add_hardware_key, commands_hardware::remove_hardware_key, commands_hardware::disable_hardware_keys,
        // Accounts and codes.
        commands::import_text, commands::list_vendors, commands::list_codes, commands::copy_code,
        commands::rename_account, commands::delete_account,
        // Settings and where the vault lives.
        commands::get_settings, commands::set_auto_lock_minutes, commands::set_capture_protection,
        commands::set_hide_codes, commands::set_tray_shows_codes, commands::set_lock_on_sleep,
        commands::set_clipboard_clear_seconds, commands::record_activity, commands::get_vault_folder,
        commands::choose_vault_folder,
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(autostart::plugin())
        .plugin(hotkey::plugin())
        .on_window_event(window::keep_running_in_tray)
        .invoke_handler(command_handler())
        .setup(setup_app)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(clear_clipboard_on_exit);
}

/// A copied code must not outlive the app on the clipboard.
fn clear_clipboard_on_exit(app: &tauri::AppHandle, event: tauri::RunEvent) {
    if matches!(event, tauri::RunEvent::Exit) {
        clipboard::clear_copied_code(app);
    }
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = data_dir_override::resolve_app_data_dir(
        app.path().app_data_dir()?,
        std::env::var_os(data_dir_override::DATA_DIR_VARIABLE),
    );
    let state = AppState::new(data_dir);
    state.replace_settings(settings_store::load_settings(&state.settings_path()));
    let capture_protection_enabled = state.settings().capture_protection;
    app.manage(state);
    if let Some(window) = app.get_webview_window("main") {
        capture_protection::apply(&window, capture_protection_enabled);
    }
    autostart::load_initial_state(app.handle());
    hotkey::register(app.handle());
    tray::create_tray(app.handle())?;
    quick_panel::create(app.handle(), capture_protection_enabled && capture_protection::is_supported())?;
    tray::spawn_refresh_loop(app.handle().clone());
    auto_lock::spawn_loop(app.handle().clone());
    system_lock_watcher::spawn_loop(app.handle().clone());
    disk_watch::spawn_loop(app.handle().clone());
    if should_show_panel(std::env::args()) {
        quick_panel::toggle(app.handle(), None);
    }
    if !should_start_hidden(std::env::args()) {
        window::show_main_window(app.handle());
    }
    Ok(())
}
