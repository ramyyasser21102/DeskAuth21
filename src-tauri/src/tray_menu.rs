//! Building the native tray menu from the model, and keeping its text current.

use crate::copy_policy::copy_item_id;
use crate::tray_model::{MenuStructure, TrayEntry, TrayModel};
use std::sync::{Mutex, PoisonError};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Wry};

pub const MENU_ID_AUTOSTART: &str = "autostart";
pub const MENU_ID_SHOW: &str = "show";
pub const MENU_ID_SETTINGS: &str = "settings";
pub const MENU_ID_LOCK: &str = "lock";
pub const MENU_ID_QUIT: &str = "quit";
const MENU_ID_STATUS: &str = "status";
const MENU_ID_COUNTDOWN: &str = "countdown";
const QUIT_ACCELERATOR: &str = "CmdOrCtrl+Q";

/// The items whose text changes every second. Cheap to clone (they are handles).
#[derive(Clone)]
pub struct LiveItems {
    countdown: Option<MenuItem<Wry>>,
    rows: Vec<MenuItem<Wry>>,
}

pub struct BuiltMenu {
    pub structure: MenuStructure,
    pub menu: Menu<Wry>,
    live_items: LiveItems,
}

/// The menu currently shown. Never hold this lock while calling into the OS: sync
/// commands run on the main thread and also refresh the tray, which would deadlock.
#[derive(Default)]
pub struct TrayMenuState(Mutex<Option<BuiltMenu>>);

impl TrayMenuState {
    pub fn live_items_if_current(&self, structure: &MenuStructure) -> Option<LiveItems> {
        let slot = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        slot.as_ref().filter(|built| &built.structure == structure).map(|built| built.live_items.clone())
    }

    pub fn store(&self, built: BuiltMenu) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(built);
    }
}

pub fn update_texts(items: &LiveItems, model: &TrayModel) -> tauri::Result<()> {
    if let (Some(item), Some(text)) = (&items.countdown, &model.countdown_text) {
        item.set_text(text)?;
    }
    for (item, row) in items.rows.iter().zip(model.account_rows()) {
        item.set_text(&row.text)?;
    }
    Ok(())
}

pub fn build_menu(app: &AppHandle, model: &TrayModel) -> tauri::Result<BuiltMenu> {
    let menu = Menu::new(app)?;
    if let Some(text) = model.placeholder {
        menu.append(&MenuItem::with_id(app, MENU_ID_STATUS, text, false, None::<&str>)?)?;
    }
    let live_items = append_entries(app, &menu, model)?;
    append_footer(app, &menu, model)?;
    Ok(BuiltMenu { structure: model.structure(), menu, live_items })
}

fn append_entries(app: &AppHandle, menu: &Menu<Wry>, model: &TrayModel) -> tauri::Result<LiveItems> {
    let countdown = match &model.countdown_text {
        Some(text) => {
            let item = MenuItem::with_id(app, MENU_ID_COUNTDOWN, text, false, None::<&str>)?;
            menu.append(&item)?;
            Some(item)
        }
        None => None,
    };
    let mut rows = Vec::new();
    for entry in &model.entries {
        match entry {
            TrayEntry::Header(text) => menu.append(&MenuItem::new(app, text, false, None::<&str>)?)?,
            TrayEntry::Account(row) => {
                let item = MenuItem::with_id(app, copy_item_id(&row.account_id), &row.text, true, None::<&str>)?;
                menu.append(&item)?;
                rows.push(item);
            }
        }
    }
    Ok(LiveItems { countdown, rows })
}

/// Open, settings, lock, and the rest. Locking is greyed out when there is nothing to lock.
fn append_footer(app: &AppHandle, menu: &Menu<Wry>, model: &TrayModel) -> tauri::Result<()> {
    if model.placeholder.is_some() || !model.entries.is_empty() {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    menu.append(&MenuItem::with_id(app, MENU_ID_SHOW, "Open DA21", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, MENU_ID_SETTINGS, "Settings", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, MENU_ID_LOCK, "Lock now", model.is_unlocked, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    let launch_at_login = CheckMenuItem::with_id(app, MENU_ID_AUTOSTART, "Launch at login", true, model.is_autostart_enabled, None::<&str>)?;
    menu.append(&launch_at_login)?;
    menu.append(&MenuItem::with_id(app, MENU_ID_QUIT, "Quit", true, Some(QUIT_ACCELERATOR))?)
}
