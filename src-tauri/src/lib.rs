mod import;
mod otp;
mod vault;

use otp::Account;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, State};
use vault::{KdfParams, Session};

const TRAY_ID: &str = "main";
const VAULT_FILE: &str = "vault.da21";
const MIN_PASSPHRASE_CHARS: usize = 8;

/// `session` is `Some` exactly when the vault is unlocked; `accounts` is empty while locked.
#[derive(Default)]
struct Inner {
    session: Option<Session>,
    accounts: Vec<Account>,
}

struct AppState {
    vault_path: PathBuf,
    inner: Mutex<Inner>,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Write via a temp file + rename so a crash mid-save never leaves a truncated vault.
fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

fn save(state: &AppState, inner: &Inner) -> Result<(), String> {
    let session = inner.session.as_ref().ok_or("vault is locked")?;
    let bytes = session.seal(&inner.accounts).map_err(|e| e.to_string())?;
    write_atomic(&state.vault_path, &bytes).map_err(|e| format!("could not write vault: {e}"))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum VaultStatus {
    Uninitialized,
    Locked,
    Unlocked,
}

#[derive(serde::Serialize)]
struct CodeView {
    label: String,
    code: String,
    seconds_left: u64,
}

#[tauri::command]
fn vault_status(state: State<AppState>) -> VaultStatus {
    if state.inner.lock().unwrap().session.is_some() {
        VaultStatus::Unlocked
    } else if state.vault_path.exists() {
        VaultStatus::Locked
    } else {
        VaultStatus::Uninitialized
    }
}

#[tauri::command]
async fn create_vault(passphrase: String, state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    if passphrase.chars().count() < MIN_PASSPHRASE_CHARS {
        return Err(format!("passphrase must be at least {MIN_PASSPHRASE_CHARS} characters"));
    }
    if state.vault_path.exists() {
        return Err("a vault already exists".into());
    }
    let session = Session::create(&passphrase, KdfParams::DEFAULT).map_err(|e| e.to_string())?;
    {
        let mut inner = state.inner.lock().unwrap();
        inner.session = Some(session);
        inner.accounts.clear();
        save(&state, &inner)?;
    }
    refresh_tray(&app);
    Ok(())
}

#[tauri::command]
async fn unlock(passphrase: String, state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let bytes = std::fs::read(&state.vault_path).map_err(|e| format!("could not read vault: {e}"))?;
    let (session, accounts) = Session::open(&bytes, &passphrase).map_err(|e| e.to_string())?;
    {
        let mut inner = state.inner.lock().unwrap();
        inner.session = Some(session);
        inner.accounts = accounts;
    }
    refresh_tray(&app);
    Ok(())
}

#[tauri::command]
fn lock(state: State<AppState>, app: AppHandle) {
    *state.inner.lock().unwrap() = Inner::default();
    refresh_tray(&app);
}

#[tauri::command]
fn import_uri(uri: String, state: State<AppState>, app: AppHandle) -> Result<usize, String> {
    let imported = import::import_uri(&uri).map_err(|e| e.to_string())?;
    let n = imported.len();
    {
        let mut inner = state.inner.lock().unwrap();
        if inner.session.is_none() {
            return Err("vault is locked".into());
        }
        let rollback = inner.accounts.len();
        inner.accounts.extend(imported);
        // Don't keep accounts in memory that failed to reach disk.
        if let Err(e) = save(&state, &inner) {
            inner.accounts.truncate(rollback);
            return Err(e);
        }
    }
    refresh_tray(&app);
    Ok(n)
}

#[tauri::command]
fn list_codes(state: State<AppState>) -> Result<Vec<CodeView>, String> {
    let inner = state.inner.lock().unwrap();
    if inner.session.is_none() {
        return Err("vault is locked".into());
    }
    let t = now();
    Ok(inner
        .accounts
        .iter()
        .map(|a| CodeView { label: a.label(), code: a.code_at(t), seconds_left: a.seconds_left(t) })
        .collect())
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    let t = now();
    {
        let state = app.state::<AppState>();
        let inner = state.inner.lock().unwrap();
        if inner.session.is_none() {
            let text = if state.vault_path.exists() { "Locked — open DeskAuth21 to unlock" } else { "Open DeskAuth21 to create your vault" };
            menu.append(&MenuItem::with_id(app, "locked", text, false, None::<&str>)?)?;
        } else if inner.accounts.is_empty() {
            menu.append(&MenuItem::with_id(app, "empty", "No accounts — open DeskAuth21 to import", false, None::<&str>)?)?;
        }
        for (i, a) in inner.accounts.iter().enumerate() {
            let c = a.code_at(t);
            let text = format!("{}  {} {}  ({}s)", a.label(), &c[..c.len() / 2], &c[c.len() / 2..], a.seconds_left(t));
            menu.append(&MenuItem::with_id(app, format!("copy:{i}"), text, true, None::<&str>)?)?;
        }
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
        .invoke_handler(tauri::generate_handler![vault_status, create_vault, unlock, lock, import_uri, list_codes])
        .setup(|app| {
            app.manage(AppState {
                vault_path: app.path().app_data_dir()?.join(VAULT_FILE),
                inner: Mutex::default(),
            });
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
                    // Clipboard copy lands in its own branch (needs the clipboard plugin).
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
