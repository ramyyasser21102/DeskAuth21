//! The single error type surfaced to the frontend.

use crate::import::ImportError;
use crate::vault::VaultError;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("vault is locked")]
    VaultLocked,
    #[error("a vault already exists")]
    VaultAlreadyExists,
    #[error("passphrase must be at least {0} characters")]
    PassphraseTooShort(usize),
    #[error(transparent)]
    Import(#[from] ImportError),
    #[error(transparent)]
    Vault(#[from] VaultError),
    #[error("could not access the vault file: {0}")]
    Storage(#[from] std::io::Error),
}

/// Tauri sends errors to JavaScript as plain strings.
impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
