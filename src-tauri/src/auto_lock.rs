//! Background check that locks the vault after the user's idle timeout.

use crate::state::AppState;
use crate::locking::clean_up_after_lock;
use crate::vault_actions::lock_if_idle;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const CHECK_INTERVAL: Duration = Duration::from_secs(1);

pub fn spawn_loop(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(CHECK_INTERVAL);
        if lock_if_idle(&app.state::<AppState>(), Instant::now()) {
            clean_up_after_lock(&app);
        }
    });
}
