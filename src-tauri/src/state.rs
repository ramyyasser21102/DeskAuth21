//! In-memory application state: whether the vault is unlocked, and its accounts.

use crate::activity::ActivityClock;
use crate::error::CommandError;
use crate::record::{AccountRecord, VaultData};
use crate::settings::Settings;
use crate::vault::Session;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Instant;

const SETTINGS_FILE_NAME: &str = "settings.json";
pub const VAULT_FILE_NAME: &str = "vault.da21";

/// The vault lives in the chosen folder, or in the app's data directory by default.
pub fn resolve_vault_path(app_data_dir: &Path, vault_folder: Option<&str>) -> PathBuf {
    let folder = vault_folder.map_or_else(|| app_data_dir.to_path_buf(), PathBuf::from);
    folder.join(VAULT_FILE_NAME)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VaultStatus {
    Uninitialized,
    Locked,
    Unlocked,
}

pub fn determine_vault_status(is_unlocked: bool, vault_file_exists: bool) -> VaultStatus {
    match (is_unlocked, vault_file_exists) {
        (true, _) => VaultStatus::Unlocked,
        (false, true) => VaultStatus::Locked,
        (false, false) => VaultStatus::Uninitialized,
    }
}

/// `session` is `Some` exactly when the vault is unlocked, and `data` is empty
/// whenever it is locked. Both change together so they cannot disagree.
#[derive(Default)]
pub struct VaultState {
    session: Option<Session>,
    data: VaultData,
    /// Identifies the vault file contents we last read or wrote, so a change made
    /// by another device can be told apart from our own saves.
    disk_fingerprint: Option<u64>,
    /// Sibling files we should leave alone until they change, by the fingerprint
    /// we saw: ones we cannot read with our key, or could not set aside after
    /// merging. Without this the same file would be retried every second.
    skipped_copies: HashMap<PathBuf, u64>,
}

impl VaultState {
    pub fn is_unlocked(&self) -> bool {
        self.session.is_some()
    }

    pub fn session(&self) -> Result<&Session, CommandError> {
        self.session.as_ref().ok_or(CommandError::VaultLocked)
    }

    pub fn data(&self) -> &VaultData {
        &self.data
    }

    pub fn accounts(&self) -> &[AccountRecord] {
        &self.data.accounts
    }

    pub fn replace_data(&mut self, data: VaultData) {
        self.data = data;
    }

    pub fn disk_fingerprint(&self) -> Option<u64> {
        self.disk_fingerprint
    }

    pub fn set_disk_fingerprint(&mut self, fingerprint: Option<u64>) {
        self.disk_fingerprint = fingerprint;
    }

    pub fn is_skipped_copy(&self, path: &Path, fingerprint: u64) -> bool {
        self.skipped_copies.get(path) == Some(&fingerprint)
    }

    pub fn skip_copy_until_it_changes(&mut self, path: &Path, fingerprint: u64) {
        self.skipped_copies.insert(path.to_path_buf(), fingerprint);
    }

    pub fn unlock(&mut self, session: Session, data: VaultData) {
        self.session = Some(session);
        self.data = data;
    }

    /// After reading a changed vault file with our key, follows the unlock slots it holds.
    pub fn adopt_slots_from(&mut self, file: &[u8]) {
        if let Some(Err(error)) = self.session.as_mut().map(|session| session.adopt_slots_from(file)) {
            eprintln!("could not follow the vault file's unlock slots: {error}");
        }
    }

    /// Swaps in a session for the same unlocked vault and hands back the one it replaced.
    pub fn replace_session(&mut self, session: Session) -> Option<Session> {
        self.session.replace(session)
    }

    /// Dropping the old state zeroizes the derived key and every account secret.
    pub fn relock(&mut self) {
        *self = Self::default();
    }
}

pub struct AppState {
    /// Fixed, so the settings file (which says where the vault is) can always be found.
    app_data_dir: PathBuf,
    /// Cached launch-at-login setting; see `autostart`.
    pub autostart_enabled: AtomicBool,
    pub activity: ActivityClock,
    settings: Mutex<Settings>,
    vault: Mutex<VaultState>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            app_data_dir,
            autostart_enabled: AtomicBool::new(false),
            activity: ActivityClock::new(Instant::now()),
            settings: Mutex::default(),
            vault: Mutex::default(),
        }
    }

    /// A poisoned lock still guards consistent data (every update is a single
    /// assignment), so recover it instead of crashing the tray thread.
    pub fn vault_guard(&self) -> MutexGuard<'_, VaultState> {
        self.vault.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn settings_path(&self) -> PathBuf {
        self.app_data_dir.join(SETTINGS_FILE_NAME)
    }

    /// Where the vault would be if `folder` were chosen.
    pub fn vault_path_in(&self, folder: &str) -> PathBuf {
        resolve_vault_path(&self.app_data_dir, Some(folder))
    }

    pub fn vault_path(&self) -> PathBuf {
        resolve_vault_path(&self.app_data_dir, self.settings().vault_folder.as_deref())
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub fn replace_settings(&self, settings: Settings) {
        *self.settings.lock().unwrap_or_else(PoisonError::into_inner) = settings;
    }

    /// Call for anything the user did on purpose (not for background polling).
    pub fn record_activity(&self) {
        self.activity.touch(Instant::now());
    }

    pub fn status_of(&self, vault: &VaultState) -> VaultStatus {
        determine_vault_status(vault.is_unlocked(), self.vault_path().exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_prefers_unlocked_then_file_presence() {
        assert_eq!(determine_vault_status(true, true), VaultStatus::Unlocked);
        assert_eq!(determine_vault_status(false, true), VaultStatus::Locked);
        assert_eq!(determine_vault_status(false, false), VaultStatus::Uninitialized);
    }

    #[test]
    fn the_vault_defaults_to_the_app_data_directory() {
        let data_dir = Path::new("/data");
        assert_eq!(resolve_vault_path(data_dir, None), data_dir.join("vault.da21"));
    }

    #[test]
    fn a_chosen_folder_replaces_the_app_data_directory() {
        let data_dir = Path::new("/data");
        assert_eq!(resolve_vault_path(data_dir, Some("/sync/Auth")), Path::new("/sync/Auth").join("vault.da21"));
    }

    #[test]
    fn a_locked_vault_has_no_session_and_reports_it() {
        let vault = VaultState::default();
        assert!(!vault.is_unlocked());
        assert!(matches!(vault.session(), Err(CommandError::VaultLocked)));
        assert!(vault.accounts().is_empty());
    }

    #[test]
    fn relocking_drops_the_session_and_every_account() {
        use crate::test_support::{device_with_account, remove_all, ACME_URI};
        let (state, directory) = device_with_account("relock", ACME_URI);
        assert_eq!(state.vault_guard().accounts().len(), 1);
        state.vault_guard().relock();
        assert!(!state.vault_guard().is_unlocked());
        assert!(state.vault_guard().accounts().is_empty());
        remove_all(&[directory]);
    }
}
