//! Keeps the unlocked vault in step with the file on disk, which another
//! device may have changed through a sync service.
//!
//! The two copies are merged per account (see `merge`), so edits and deletes
//! made on different devices resolve the same way everywhere.

use crate::conflict_copies::merge_conflict_copies;
use crate::merge::{has_same_content, merge};
use crate::persistence::{fingerprint, read_bounded, save_vault, MAX_VAULT_FILE_BYTES};
use crate::record::VaultData;
use crate::state::VaultState;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskSyncOutcome {
    /// Nothing to do: the file is ours, absent, or holds nothing we lack.
    Unchanged,
    /// Changes were merged in, or this device's changes were written back.
    Updated,
    /// The file changed but cannot be read with our key: a different vault, or
    /// a copy still being written by the sync service.
    Unreadable,
}

/// Everything another device may have left for us: changes to the vault file,
/// and conflicted copies the sync service made when two devices wrote at once.
pub fn sync_vault_folder(path: &Path, vault: &mut VaultState) -> DiskSyncOutcome {
    let file_outcome = sync_from_disk(path, vault);
    if file_outcome == DiskSyncOutcome::Unreadable {
        return file_outcome;
    }
    let merged_copies = merge_conflict_copies(path, vault);
    if merged_copies > 0 || file_outcome == DiskSyncOutcome::Updated {
        DiskSyncOutcome::Updated
    } else {
        DiskSyncOutcome::Unchanged
    }
}

/// Merges the file into memory and, if the file lacked something we hold (for
/// example a sync conflict restored an older copy), writes the merge back.
pub fn sync_from_disk(path: &Path, vault: &mut VaultState) -> DiskSyncOutcome {
    if !vault.is_unlocked() {
        return DiskSyncOutcome::Unchanged;
    }
    // A missing or unreadable file has nothing to merge; the next save recreates it.
    let Ok(file_bytes) = read_bounded(path, MAX_VAULT_FILE_BYTES) else {
        return DiskSyncOutcome::Unchanged;
    };
    let seen = fingerprint(&file_bytes);
    if vault.disk_fingerprint() == Some(seen) {
        return DiskSyncOutcome::Unchanged;
    }
    let Some(disk_data) = vault.session().ok().and_then(|session| session.unseal(&file_bytes).ok()) else {
        return DiskSyncOutcome::Unreadable;
    };
    merge_disk_data(path, vault, &disk_data, seen)
}

fn merge_disk_data(path: &Path, vault: &mut VaultState, disk_data: &VaultData, seen: u64) -> DiskSyncOutcome {
    let merged = merge(vault.data(), disk_data);
    let is_changed_here = !has_same_content(&merged, vault.data());
    let disk_lacks_something = !has_same_content(&merged, disk_data);
    vault.replace_data(merged);
    vault.set_disk_fingerprint(Some(seen));
    if disk_lacks_something {
        write_back(path, vault);
    }
    if is_changed_here || disk_lacks_something { DiskSyncOutcome::Updated } else { DiskSyncOutcome::Unchanged }
}

fn write_back(path: &Path, vault: &mut VaultState) {
    if let Err(error) = save_vault(path, vault) {
        eprintln!("could not write the merged vault back: {error}");
    }
}

#[cfg(test)]
mod tests;
