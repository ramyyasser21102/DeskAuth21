//! Keeps the unlocked vault in step with the file on disk, which another
//! device may have changed through a sync service.
//!
//! Accounts are only ever added, so merging is a union: it gives the same
//! result in either order, and two devices can never undo each other's work.

use crate::dedupe::split_new_accounts;
use crate::persistence::{fingerprint, save_vault};
use crate::state::VaultState;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskSyncOutcome {
    /// Nothing to do: the file is ours, absent, or holds nothing we lack.
    Unchanged,
    /// Accounts were merged in, or this device's accounts were written back.
    Updated,
    /// The file changed but cannot be read with our key: a different vault, or
    /// a copy still being written by the sync service.
    Unreadable,
}

/// Merges the file's accounts into memory and, if the file lacked accounts we
/// hold (for example a sync conflict restored an older copy), writes them back.
pub fn sync_from_disk(path: &Path, vault: &mut VaultState) -> DiskSyncOutcome {
    if !vault.is_unlocked() {
        return DiskSyncOutcome::Unchanged;
    }
    // A missing or unreadable file has nothing to merge; the next save recreates it.
    let Ok(file_bytes) = std::fs::read(path) else {
        return DiskSyncOutcome::Unchanged;
    };
    let seen = fingerprint(&file_bytes);
    if vault.disk_fingerprint() == Some(seen) {
        return DiskSyncOutcome::Unchanged;
    }
    let Some(disk_accounts) = vault.session().ok().and_then(|session| session.unseal(&file_bytes).ok()) else {
        return DiskSyncOutcome::Unreadable;
    };
    merge_disk_accounts(path, vault, disk_accounts, seen)
}

fn merge_disk_accounts(
    path: &Path,
    vault: &mut VaultState,
    disk_accounts: Vec<crate::otp::Account>,
    seen: u64,
) -> DiskSyncOutcome {
    let (from_disk, _) = split_new_accounts(vault.accounts(), disk_accounts.clone());
    let (only_here, _) = split_new_accounts(&disk_accounts, vault.accounts().to_vec());
    let is_changed = !from_disk.is_empty() || !only_here.is_empty();
    vault.append_accounts(from_disk);
    vault.set_disk_fingerprint(Some(seen));
    if !only_here.is_empty() {
        write_back(path, vault);
    }
    if is_changed { DiskSyncOutcome::Updated } else { DiskSyncOutcome::Unchanged }
}

fn write_back(path: &Path, vault: &mut VaultState) {
    if let Err(error) = save_vault(path, vault) {
        eprintln!("could not write the merged vault back: {error}");
    }
}

#[cfg(test)]
mod tests;
