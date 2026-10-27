//! The menu-bar / system-tray icon and its menu.
//!
//! The OS closes an open menu when it is replaced, so the menu is kept and its
//! items' text is updated in place once a second. It is only rebuilt when the
//! structure changes (accounts added or removed, locked or unlocked).

use crate::autostart;
use crate::clipboard::copy_code_for_account;
use crate::clock::current_unix_seconds;
use crate::codes::build_code_views;
use crate::copy_policy::{copy_item_id, parse_copy_item_id};
use crate::state::AppState;
use crate::tray_model::{build_tray_model, more_accounts_text, MenuStructure, TrayModel};
use crate::window::show_main_window;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, PoisonError};
use std::time::Duration;
use tauri::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::image::Image;
use tauri::{AppHandle, Manager, Wry};

const TRAY_ID: &str = "main";
const MENU_BAR_TEMPLATE_PNG: &[u8] = include_bytes!("../icons/tray-icon.png");
const MENU_ID_STATUS: &str = "status";
const MENU_ID_COUNTDOWN: &str = "countdown";
const MENU_ID_MORE: &str = "more";
const MENU_ID_AUTOSTART: &str = "autostart";
const MENU_ID_SHOW: &str = "show";
const MENU_ID_QUIT: &str = "quit";
const QUIT_ACCELERATOR: &str = "CmdOrCtrl+Q";
const EXIT_SUCCESS: i32 = 0;
const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

/// The items whose text changes every second. Cheap to clone (they are handles).
#[derive(Clone)]
struct LiveItems {
    countdown: Option<MenuItem<Wry>>,
    rows: Vec<MenuItem<Wry>>,
}

struct BuiltMenu {
    structure: MenuStructure,
    menu: Menu<Wry>,
    live_items: LiveItems,
}

/// The menu currently shown. Never hold this lock while calling into the OS: sync
/// commands run on the main thread and also refresh the tray, which would deadlock.
#[derive(Default)]
pub struct TrayMenuState(Mutex<Option<BuiltMenu>>);

impl TrayMenuState {
    fn live_items_if_current(&self, structure: &MenuStructure) -> Option<LiveItems> {
        let slot = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        slot.as_ref().filter(|built| &built.structure == structure).map(|built| built.live_items.clone())
    }

    fn store(&self, built: BuiltMenu) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(built);
    }
}

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
    build_tray_model(state.status_of(&vault), &views, state.autostart_enabled.load(Ordering::Relaxed), state.settings().tray_shows_codes)
}

fn update_texts(items: &LiveItems, model: &TrayModel) -> tauri::Result<()> {
    if let (Some(item), Some(text)) = (&items.countdown, &model.countdown_text) {
        item.set_text(text)?;
    }
    for (item, row) in items.rows.iter().zip(&model.rows) {
        item.set_text(&row.text)?;
    }
    Ok(())
}

fn replace_menu(app: &AppHandle, tray: &TrayIcon<Wry>, model: &TrayModel) -> tauri::Result<()> {
    let built = build_menu(app, model)?;
    tray.set_menu(Some(built.menu.clone()))?;
    app.state::<TrayMenuState>().store(built);
    Ok(())
}

fn build_menu(app: &AppHandle, model: &TrayModel) -> tauri::Result<BuiltMenu> {
    let menu = Menu::new(app)?;
    if let Some(text) = model.placeholder {
        menu.append(&MenuItem::with_id(app, MENU_ID_STATUS, text, false, None::<&str>)?)?;
    }
    let live_items = append_live_items(app, &menu, model)?;
    if model.hidden_accounts > 0 {
        let more = MenuItem::with_id(app, MENU_ID_MORE, more_accounts_text(model.hidden_accounts), true, None::<&str>)?;
        menu.append(&more)?;
    }
    append_footer(app, &menu, model.is_autostart_enabled)?;
    Ok(BuiltMenu { structure: model.structure(), menu, live_items })
}

fn append_live_items(app: &AppHandle, menu: &Menu<Wry>, model: &TrayModel) -> tauri::Result<LiveItems> {
    let countdown = match &model.countdown_text {
        Some(text) => {
            let item = MenuItem::with_id(app, MENU_ID_COUNTDOWN, text, false, None::<&str>)?;
            menu.append(&item)?;
            Some(item)
        }
        None => None,
    };
    let mut rows = Vec::with_capacity(model.rows.len());
    for row in &model.rows {
        let item = MenuItem::with_id(app, copy_item_id(&row.account_id), &row.text, true, None::<&str>)?;
        menu.append(&item)?;
        rows.push(item);
    }
    Ok(LiveItems { countdown, rows })
}

fn append_footer(app: &AppHandle, menu: &Menu<Wry>, is_autostart_enabled: bool) -> tauri::Result<()> {
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    let launch_at_login = CheckMenuItem::with_id(app, MENU_ID_AUTOSTART, "Launch at login", true, is_autostart_enabled, None::<&str>)?;
    menu.append(&launch_at_login)?;
    menu.append(&MenuItem::with_id(app, MENU_ID_SHOW, "Open DeskAuth21", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, MENU_ID_QUIT, "Quit", true, Some(QUIT_ACCELERATOR))?)
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        MENU_ID_QUIT => app.exit(EXIT_SUCCESS),
        MENU_ID_SHOW | MENU_ID_MORE => show_main_window(app),
        MENU_ID_AUTOSTART => toggle_autostart_from_menu(app),
        item_id => copy_from_menu_item(app, item_id),
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
