//! Locks the vault when the computer sleeps or the screen locks, if the person asked for that.

use crate::clock::current_unix_millis;
use crate::lock_triggers::{should_lock, ScreenLockDetector, SuspendDetector};
use crate::locking::lock_and_clean_up;
use crate::screen_lock::is_screen_locked;
use crate::state::AppState;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const CHECK_INTERVAL: Duration = Duration::from_secs(1);

/// Readings are taken every second even while the setting is off, so switching it on
/// never reacts to something that happened earlier.
struct Watcher {
    started: Instant,
    suspend: SuspendDetector,
    screen: ScreenLockDetector,
}

impl Watcher {
    fn new() -> Self {
        Self { started: Instant::now(), suspend: SuspendDetector::default(), screen: ScreenLockDetector::default() }
    }

    fn should_lock_now(&mut self, lock_on_sleep_enabled: bool) -> bool {
        let suspended = self.suspend.observe(u128::from(current_unix_millis()), self.started.elapsed().as_millis());
        let screen_just_locked = self.screen.observe(is_screen_locked());
        should_lock(lock_on_sleep_enabled, suspended, screen_just_locked)
    }
}

pub fn spawn_loop(app: AppHandle) {
    std::thread::spawn(move || watch_forever(&app));
}

fn watch_forever(app: &AppHandle) {
    let mut watcher = Watcher::new();
    loop {
        std::thread::sleep(CHECK_INTERVAL);
        let is_enabled = app.state::<AppState>().settings().lock_on_sleep;
        if watcher.should_lock_now(is_enabled) {
            lock_and_clean_up(app);
        }
    }
}
