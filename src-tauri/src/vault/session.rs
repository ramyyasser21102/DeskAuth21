//! An unlocked vault: the keys needed to read and re-seal it, in whichever file
//! format it was opened. Vaults made before slots existed stay in the old format
//! until the person upgrades them.

use super::layout::{file_version, VERSION_1, VERSION_2};
use super::legacy::LegacyKeys;
use super::slotted::SlottedKeys;
use super::{KdfParams, VaultError};
use crate::record::VaultData;

enum Keys {
    Legacy(LegacyKeys),
    Slotted(SlottedKeys),
}

pub struct Session {
    keys: Keys,
}

impl Session {
    /// A brand-new vault, in the current format.
    pub fn create(passphrase: &str, params: KdfParams) -> Result<Self, VaultError> {
        Ok(Self { keys: Keys::Slotted(SlottedKeys::create(passphrase, params)?) })
    }

    pub fn open(bytes: &[u8], passphrase: &str) -> Result<(Self, VaultData), VaultError> {
        match file_version(bytes) {
            Some(VERSION_1) => LegacyKeys::open(bytes, passphrase).map(|(keys, data)| (Self { keys: Keys::Legacy(keys) }, data)),
            Some(VERSION_2) => SlottedKeys::open(bytes, passphrase).map(|(keys, data)| (Self { keys: Keys::Slotted(keys) }, data)),
            _ => Err(VaultError::BadFormat),
        }
    }

    pub fn seal(&self, data: &VaultData) -> Result<Vec<u8>, VaultError> {
        match &self.keys {
            Keys::Legacy(keys) => keys.seal(data),
            Keys::Slotted(keys) => keys.seal(data),
        }
    }

    /// Decrypts another copy of this vault. A copy in a different format than ours is
    /// refused rather than guessed at, so nothing is overwritten by mistake.
    pub fn unseal(&self, file: &[u8]) -> Result<VaultData, VaultError> {
        match (&self.keys, file_version(file)) {
            (Keys::Legacy(keys), Some(VERSION_1)) => keys.unseal(file),
            (Keys::Slotted(keys), Some(VERSION_2)) => keys.unseal(file),
            (Keys::Legacy(_), Some(VERSION_2)) => Err(VaultError::NewerFormat),
            (Keys::Slotted(_), Some(VERSION_1)) => Err(VaultError::OlderFormat),
            _ => Err(VaultError::BadFormat),
        }
    }

    pub fn format_version(&self) -> u8 {
        match self.keys {
            Keys::Legacy(_) => VERSION_1,
            Keys::Slotted(_) => VERSION_2,
        }
    }

    /// The same vault in the current format, or `None` if it already is. The caller
    /// saves it and swaps it in.
    pub fn upgraded(&self) -> Result<Option<Self>, VaultError> {
        match &self.keys {
            Keys::Legacy(legacy) => Ok(Some(Self { keys: Keys::Slotted(SlottedKeys::from_legacy(legacy)?) })),
            Keys::Slotted(_) => Ok(None),
        }
    }
}
