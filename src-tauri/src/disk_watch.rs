//! Background check that picks up changes other devices make to the vault file.

use crate::disk_sync::{sync_vault_folder, DiskSyncOutcome};
use crate::state::AppState;
use crate::tray::refresh_tray;
use std::time::Duration;
use tauri::{AppHandle, Manager};

const CHECK_INTERVAL: Duration = Duration::from_secs(1);

pub fn spawn_loop(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(CHECK_INTERVAL);
        if sync_once(&app) == DiskSyncOutcome::Updated {
            refresh_tray(&app);
        }
    });
}

fn sync_once(app: &AppHandle) -> DiskSyncOutcome {
    let state = app.state::<AppState>();
    let mut vault = state.vault_guard();
    sync_vault_folder(&state.vault_path(), &mut vault)
}
