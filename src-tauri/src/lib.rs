mod import;
mod otp;
mod vault;

use otp::Account;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State};

const TRAY_ID: &str = "main";

#[derive(Default)]
struct Accounts(Mutex<Vec<Account>>);

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

#[derive(serde::Serialize)]
struct CodeView {
    label: String,
    code: String,
    seconds_left: u64,
}

#[tauri::command]
fn import_uri(uri: String, state: State<Accounts>, app: AppHandle) -> Result<usize, String> {
    let imported = import::import_uri(&uri).map_err(|e| e.to_string())?;
    let n = imported.len();
    state.0.lock().unwrap().extend(imported);
    refresh_tray(&app);
    Ok(n)
}

#[tauri::command]
fn list_codes(state: State<Accounts>) -> Vec<CodeView> {
    let t = now();
    state
        .0
        .lock()
        .unwrap()
        .iter()
        .map(|a| CodeView { label: a.label(), code: a.code_at(t), seconds_left: a.seconds_left(t) })
        .collect()
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    let t = now();
    let accounts = app.state::<Accounts>();
    let accounts = accounts.0.lock().unwrap();
    if accounts.is_empty() {
        menu.append(&MenuItem::with_id(app, "empty", "No accounts — open DeskAuth21 to import", false, None::<&str>)?)?;
    }
    for (i, a) in accounts.iter().enumerate() {
        let c = a.code_at(t);
        let text = format!("{}  {} {}  ({}s)", a.label(), &c[..c.len() / 2], &c[c.len() / 2..], a.seconds_left(t));
        menu.append(&MenuItem::with_id(app, format!("copy:{i}"), text, true, None::<&str>)?)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "show", "Open DeskAuth21", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?)?;
    Ok(menu)
}

fn refresh_tray(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY_ID), build_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Accounts::default())
        .invoke_handler(tauri::generate_handler![import_uri, list_codes])
        .setup(|app| {
            let handle = app.handle().clone();
            TrayIconBuilder::with_id(TRAY_ID)
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&build_menu(&handle)?)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    // Clipboard copy lands with the vault milestone (needs the clipboard plugin).
                    _ => {}
                })
                .build(app)?;
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                refresh_tray(&handle);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
