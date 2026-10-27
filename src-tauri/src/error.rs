//! The single error type surfaced to the frontend.

use crate::account_changes::ChangeError;
use crate::import::ImportError;
use crate::vault::VaultError;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("the vault is locked; unlock it and try again")]
    VaultLocked,
    #[error("a vault already exists")]
    VaultAlreadyExists,
    #[error(transparent)]
    Change(#[from] ChangeError),
    #[error("the vault file was replaced by a different vault, possibly on another device; nothing was overwritten, so lock and unlock to reload")]
    VaultChangedElsewhere,
    #[error("{0} is not an allowed setting")]
    InvalidSetting(String),
    #[error("could not change the launch-at-login setting: {0}")]
    Autostart(String),
    #[error("clipboard unavailable: {0}")]
    Clipboard(String),
    #[error("the passphrase must be at least {0} characters; add a few more")]
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
