//! Reading and writing the vault file.

use crate::error::CommandError;
use crate::record::VaultData;
use crate::state::VaultState;
use std::fs::OpenOptions;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const TEMP_SUFFIX: &str = "tmp";
const TEMP_NAME_RANDOM_BYTES: usize = 8;
#[cfg(unix)]
const OWNER_READ_WRITE_ONLY: u32 = 0o600;

/// A real vault is a few kilobytes. This leaves room for thousands of accounts while
/// stopping a hostile or runaway file in a shared folder from filling memory.
pub const MAX_VAULT_FILE_BYTES: u64 = 16 * 1024 * 1024;

pub fn read_vault_file(path: &Path) -> Result<Vec<u8>, CommandError> {
    Ok(read_bounded(path, MAX_VAULT_FILE_BYTES)?)
}

/// Reads a regular file no larger than `limit`. Anything else (a device such as
/// `/dev/zero` reached through a link, a directory, an oversized file) is refused
/// rather than read.
pub fn read_bounded(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    let metadata = std::fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "not a regular file within the size limit"));
    }
    let mut bytes = Vec::new();
    // The file may grow after the check above, so the read itself is capped too.
    std::fs::File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "file grew past the size limit"));
    }
    Ok(bytes)
}

/// A cheap identity for file contents, used only to notice that a file changed.
/// Not a security measure.
pub fn fingerprint(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Saves and remembers what is now on disk, so our own write is not mistaken
/// for a change made elsewhere.
pub fn save_vault(path: &Path, vault: &mut VaultState) -> Result<(), CommandError> {
    let sealed = vault.session()?.seal(vault.data())?;
    write_atomically(path, &sealed)?;
    vault.set_disk_fingerprint(Some(fingerprint(&sealed)));
    Ok(())
}

/// Replaces the vault contents and saves. If saving fails the previous contents
/// are restored, so memory never holds changes that did not reach disk.
pub fn replace_and_save(path: &Path, vault: &mut VaultState, new_data: VaultData) -> Result<(), CommandError> {
    vault.session()?;
    let previous = vault.data().clone();
    vault.replace_data(new_data);
    let saved = save_vault(path, vault);
    if saved.is_err() {
        vault.replace_data(previous);
    }
    saved
}

/// Write to a private temp file, flush it to disk, then rename, so a crash mid-save
/// never leaves a truncated vault. The temp name is random and created exclusively, so
/// a file or link planted in a shared folder at a guessable name cannot redirect the write.
pub fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)?;
    }
    let temp_path = unique_temp_path(path)?;
    let result = write_new_private_file(&temp_path, bytes).and_then(|()| std::fs::rename(&temp_path, path));
    if result.is_err() {
        // Best effort: the original error is what matters to the caller.
        let _ = std::fs::remove_file(&temp_path);
    }
    result
}

fn unique_temp_path(path: &Path) -> std::io::Result<PathBuf> {
    let mut random = [0u8; TEMP_NAME_RANDOM_BYTES];
    getrandom::fill(&mut random).map_err(std::io::Error::other)?;
    let suffix: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    let name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(path.with_file_name(format!(".{name}.{suffix}.{TEMP_SUFFIX}")))
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, OWNER_READ_WRITE_ONLY);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests;
