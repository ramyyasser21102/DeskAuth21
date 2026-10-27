//! Sync services leave extra copies of a file beside it when two devices wrote
//! at nearly the same time. This finds those next to the vault and merges them,
//! so neither device's changes are lost.

use crate::merge::{has_same_content, merge};
use crate::persistence::{fingerprint, read_bounded, replace_and_save, MAX_VAULT_FILE_BYTES};
use crate::state::VaultState;
use std::path::{Path, PathBuf};

const VAULT_EXTENSION: &str = ".da21";
const MERGED_SUFFIX: &str = ".merged";

/// Dropbox: `vault (Ann's conflicted copy 2026-10-27).da21`. Syncthing:
/// `vault.sync-conflict-20261027-101500-ABCDEFG.da21`. iCloud: `vault 2.da21`.
/// Google Drive: `vault (1).da21`. OneDrive: `vault-LAPTOP.da21`.
/// What they share: the vault's own name at the start and `.da21` at the end.
pub fn is_conflict_copy_name(candidate: &str, vault_file_name: &str) -> bool {
    let vault_stem = vault_file_name.strip_suffix(VAULT_EXTENSION).unwrap_or(vault_file_name);
    candidate != vault_file_name && candidate.starts_with(vault_stem) && candidate.ends_with(VAULT_EXTENSION)
}

fn find_conflict_copies(vault_path: &Path) -> Vec<PathBuf> {
    let (Some(directory), Some(vault_name)) = (vault_path.parent(), vault_path.file_name().and_then(|name| name.to_str())) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut copies: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.file_name().and_then(|name| name.to_str()).is_some_and(|name| is_conflict_copy_name(name, vault_name)))
        .collect();
    copies.sort();
    copies
}

/// Kept rather than deleted, so a merge can always be inspected or undone by hand.
fn set_aside(copy_path: &Path) -> bool {
    let mut merged_name = copy_path.as_os_str().to_os_string();
    merged_name.push(MERGED_SUFFIX);
    match std::fs::rename(copy_path, PathBuf::from(merged_name)) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("merged a conflicted copy but could not set it aside: {error}");
            false
        }
    }
}

/// Merges one copy if it decrypts with our key. Returns whether it was merged.
fn merge_one_copy(copy_path: &Path, vault_path: &Path, vault: &mut VaultState) -> bool {
    let Ok(bytes) = read_bounded(copy_path, MAX_VAULT_FILE_BYTES) else {
        return false;
    };
    let seen = fingerprint(&bytes);
    if vault.is_skipped_copy(copy_path, seen) {
        return false;
    }
    // A different vault, or a file the sync service is still writing: leave it alone.
    let Some(copy_data) = vault.session().ok().and_then(|session| session.unseal(&bytes).ok()) else {
        vault.skip_copy_until_it_changes(copy_path, seen);
        return false;
    };
    let merged = merge(vault.data(), &copy_data);
    if !has_same_content(&merged, vault.data()) && replace_and_save(vault_path, vault, merged).is_err() {
        return false;
    }
    if !set_aside(copy_path) {
        vault.skip_copy_until_it_changes(copy_path, seen);
    }
    true
}

/// Merges every readable conflicted copy beside the vault. Returns how many were merged.
pub fn merge_conflict_copies(vault_path: &Path, vault: &mut VaultState) -> usize {
    if !vault.is_unlocked() {
        return 0;
    }
    let copies = find_conflict_copies(vault_path);
    copies.iter().filter(|copy| merge_one_copy(copy, vault_path, vault)).count()
}

#[cfg(test)]
mod tests;
