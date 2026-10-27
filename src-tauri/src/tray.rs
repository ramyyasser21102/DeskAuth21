//! The menu-bar / system-tray icon and its menu.
//!
//! The OS closes an open menu when it is replaced, so the menu is kept and its
//! items' text is updated in place once a second. It is only rebuilt when the
//! structure changes (accounts added or removed, locked or unlocked).

use crate::autostart;
use crate::clipboard::copy_code_for_account;
use crate::clock::current_unix_seconds;
use crate::codes::build_code_views;
use crate::commands_panel::open_page;
use crate::copy_policy::parse_copy_item_id;
use crate::locking::lock_and_clean_up;
use crate::quick_panel;
use crate::state::AppState;
use crate::tray_menu::{build_menu, update_texts, TrayMenuState, MENU_ID_AUTOSTART, MENU_ID_LOCK, MENU_ID_PANEL, MENU_ID_QUIT, MENU_ID_SETTINGS, MENU_ID_SHOW};
use crate::tray_model::{build_tray_model, TrayModel, TrayOptions};
use crate::window::show_main_window;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::image::Image;
use tauri::menu::MenuEvent;
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager, Wry};

const TRAY_ID: &str = "main";
const MENU_BAR_TEMPLATE_PNG: &[u8] = include_bytes!("../icons/tray-icon.png");
const EXIT_SUCCESS: i32 = 0;
const REFRESH_INTERVAL: Duration = Duration::from_secs(1);
/// Linux tray icons cannot report clicks, so there the menu has to carry the accounts.
pub const LISTS_ACCOUNTS_IN_MENU: bool = cfg!(target_os = "linux");

/// The menu bar tints a "template" image to suit light or dark mode, so macOS gets the
/// plain shield. Windows and Linux trays cannot do that and may be light or dark, so they
/// keep the app icon, whose white tile reads on either.
fn tray_icon(app: &AppHandle) -> tauri::Result<Image<'static>> {
    if cfg!(target_os = "macos") {
        return Image::from_bytes(MENU_BAR_TEMPLATE_PNG);
    }
    let window_icon = app.default_window_icon().expect("tauri.conf.json must define an app icon");
    Ok(window_icon.clone().to_owned())
}

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    app.manage(TrayMenuState::default());
    let icon = tray_icon(app)?;
    let built = build_menu(app, &take_model(app))?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        .menu(&built.menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(quick_panel::handle_tray_event)
        .on_menu_event(handle_menu_event)
        .build(app)?;
    app.state::<TrayMenuState>().store(built);
    Ok(())
}

pub fn spawn_refresh_loop(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(REFRESH_INTERVAL);
        refresh_tray(&app);
    });
}

pub fn refresh_tray(app: &AppHandle) {
    if let Err(error) = try_refresh_tray(app) {
        eprintln!("could not refresh the tray menu: {error}");
    }
}

fn try_refresh_tray(app: &AppHandle) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    let model = take_model(app);
    match app.state::<TrayMenuState>().live_items_if_current(&model.structure()) {
        Some(items) => update_texts(&items, &model),
        None => replace_menu(app, &tray, &model),
    }
}

fn take_model(app: &AppHandle) -> TrayModel {
    let state = app.state::<AppState>();
    let vault = state.vault_guard();
    let views = build_code_views(vault.accounts(), current_unix_seconds());
    let options = TrayOptions {
        is_autostart_enabled: state.autostart_enabled.load(Ordering::Relaxed),
        shows_codes: state.settings().tray_shows_codes,
        lists_accounts: LISTS_ACCOUNTS_IN_MENU,
    };
    build_tray_model(state.status_of(&vault), &views, options)
}

fn replace_menu(app: &AppHandle, tray: &TrayIcon<Wry>, model: &TrayModel) -> tauri::Result<()> {
    let built = build_menu(app, model)?;
    tray.set_menu(Some(built.menu.clone()))?;
    app.state::<TrayMenuState>().store(built);
    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        MENU_ID_QUIT => app.exit(EXIT_SUCCESS),
        MENU_ID_SHOW => show_main_window(app),
        MENU_ID_SETTINGS => open_settings(app),
        MENU_ID_LOCK => lock_and_clean_up(app),
        MENU_ID_PANEL => quick_panel::toggle(app, None),
        MENU_ID_AUTOSTART => toggle_autostart_from_menu(app),
        item_id => copy_from_menu_item(app, item_id),
    }
}

fn open_settings(app: &AppHandle) {
    if let Err(error) = open_page(app, "settings") {
        eprintln!("could not open the settings: {error}");
    }
}

fn copy_from_menu_item(app: &AppHandle, item_id: &str) {
    let Some(account_id) = parse_copy_item_id(item_id) else {
        return;
    };
    if let Err(error) = copy_code_for_account(app, account_id) {
        eprintln!("could not copy code: {error}");
    }
}

fn toggle_autostart_from_menu(app: &AppHandle) {
    if let Err(error) = autostart::toggle(app) {
        eprintln!("{error}");
    }
    refresh_tray(app);
}
