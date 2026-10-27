//! The menu-bar / system-tray icon and its live-updating menu.

use crate::clock::current_unix_seconds;
use crate::codes::{build_code_views, tray_menu_text, CodeView};
use crate::state::{AppState, VaultStatus};
use std::time::Duration;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

const TRAY_ID: &str = "main";
const MAIN_WINDOW_LABEL: &str = "main";
const MENU_ID_STATUS: &str = "status";
const MENU_ID_SHOW: &str = "show";
const MENU_ID_QUIT: &str = "quit";
const COPY_ITEM_ID_PREFIX: &str = "copy:";
const QUIT_ACCELERATOR: &str = "CmdOrCtrl+Q";
const EXIT_SUCCESS: i32 = 0;
const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

struct TraySnapshot {
    status: VaultStatus,
    code_views: Vec<CodeView>,
}

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let icon = app.default_window_icon().cloned().expect("tauri.conf.json must define an app icon");
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&build_menu(app)?)
        .on_menu_event(handle_menu_event)
        .build(app)?;
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
    match app.tray_by_id(TRAY_ID) {
        Some(tray) => tray.set_menu(Some(build_menu(app)?)),
        None => Ok(()),
    }
}

fn take_snapshot(app: &AppHandle) -> TraySnapshot {
    let state = app.state::<AppState>();
    let vault = state.vault_guard();
    TraySnapshot {
        status: state.status_of(&vault),
        code_views: build_code_views(vault.accounts(), current_unix_seconds()),
    }
}

/// Text for the disabled placeholder row, when there are no codes to show.
fn placeholder_text(status: VaultStatus, has_no_accounts: bool) -> Option<&'static str> {
    match status {
        VaultStatus::Uninitialized => Some("Open DeskAuth21 to create your vault"),
        VaultStatus::Locked => Some("Locked — open DeskAuth21 to unlock"),
        VaultStatus::Unlocked if has_no_accounts => Some("No accounts — open DeskAuth21 to import"),
        VaultStatus::Unlocked => None,
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let snapshot = take_snapshot(app);
    let menu = Menu::new(app)?;
    append_placeholder(app, &menu, &snapshot)?;
    append_account_items(app, &menu, &snapshot)?;
    append_footer(app, &menu)?;
    Ok(menu)
}

fn append_placeholder(app: &AppHandle, menu: &Menu<Wry>, snapshot: &TraySnapshot) -> tauri::Result<()> {
    let text = placeholder_text(snapshot.status, snapshot.code_views.is_empty());
    match text {
        Some(text) => menu.append(&MenuItem::with_id(app, MENU_ID_STATUS, text, false, None::<&str>)?),
        None => Ok(()),
    }
}

fn append_account_items(app: &AppHandle, menu: &Menu<Wry>, snapshot: &TraySnapshot) -> tauri::Result<()> {
    for (index, view) in snapshot.code_views.iter().enumerate() {
        let item_id = format!("{COPY_ITEM_ID_PREFIX}{index}");
        menu.append(&MenuItem::with_id(app, item_id, tray_menu_text(view), true, None::<&str>)?)?;
    }
    Ok(())
}

fn append_footer(app: &AppHandle, menu: &Menu<Wry>) -> tauri::Result<()> {
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, MENU_ID_SHOW, "Open DeskAuth21", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, MENU_ID_QUIT, "Quit", true, Some(QUIT_ACCELERATOR))?)
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        MENU_ID_QUIT => app.exit(EXIT_SUCCESS),
        MENU_ID_SHOW => show_main_window(app),
        // Copy-to-clipboard items are handled in the clipboard branch.
        _ => {}
    }
}

fn show_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
    };
    // Showing can only fail if the window is being destroyed, when there is nothing to show.
    let _ = window.show();
    let _ = window.set_focus();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_explains_why_no_codes_are_shown() {
        assert!(placeholder_text(VaultStatus::Uninitialized, true).unwrap().contains("create"));
        assert!(placeholder_text(VaultStatus::Locked, true).unwrap().contains("unlock"));
        assert!(placeholder_text(VaultStatus::Unlocked, true).unwrap().contains("import"));
    }

    #[test]
    fn no_placeholder_when_codes_are_listed() {
        assert_eq!(placeholder_text(VaultStatus::Unlocked, false), None);
    }
}
