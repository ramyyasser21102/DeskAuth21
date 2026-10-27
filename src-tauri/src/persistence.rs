//! Reading and writing the vault file.

use crate::error::CommandError;
use crate::otp::Account;
use crate::state::VaultState;
use std::path::Path;

const TEMP_EXTENSION: &str = "tmp";

pub fn read_vault_file(path: &Path) -> Result<Vec<u8>, CommandError> {
    Ok(std::fs::read(path)?)
}

pub fn save_vault(path: &Path, vault: &VaultState) -> Result<(), CommandError> {
    let sealed = vault.session()?.seal(vault.accounts())?;
    write_atomically(path, &sealed)?;
    Ok(())
}

/// Adds accounts and saves. If saving fails the accounts are removed again, so
/// memory never holds accounts that did not reach disk.
pub fn append_and_save(path: &Path, vault: &mut VaultState, imported: Vec<Account>) -> Result<(), CommandError> {
    vault.session()?;
    let previous_count = vault.append_accounts(imported);
    let saved = save_vault(path, vault);
    if saved.is_err() {
        vault.truncate_accounts(previous_count);
    }
    saved
}

/// Write to a temp file then rename, so a crash mid-save never leaves a truncated vault.
fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)?;
    }
    let temp_path = path.with_extension(TEMP_EXTENSION);
    std::fs::write(&temp_path, bytes)?;
    std::fs::rename(&temp_path, path)
}
