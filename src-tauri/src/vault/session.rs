//! An unlocked vault: the keys needed to read and re-seal it, in whichever file
//! format it was opened. Vaults made before slots existed stay in the old format
//! until the person upgrades them.

use super::layout::{file_version, VERSION_1, VERSION_2};
use super::hardware_ops::HardwareKeyInfo;
use super::legacy::LegacyKeys;
use super::slotted::SlottedKeys;
use super::{KdfParams, VaultError};
use crate::key_device::{KeyDevice, KeyRegistration};
#[cfg(test)]
use crate::key_device::NoKeySupport;
use crate::record::VaultData;

#[derive(Clone)]
enum Keys {
    Legacy(LegacyKeys),
    Slotted(SlottedKeys),
}

#[derive(Clone)]
pub struct Session {
    keys: Keys,
}

impl Session {
    /// A brand-new vault, in the current format.
    pub fn create(passphrase: &str, params: KdfParams) -> Result<Self, VaultError> {
        Ok(Self { keys: Keys::Slotted(SlottedKeys::create(passphrase, params)?) })
    }

    /// For a vault that needs no hardware key; one that does fails with "not supported".
    #[cfg(test)]
    pub fn open(bytes: &[u8], passphrase: &str) -> Result<(Self, VaultData), VaultError> {
        Self::open_with_key(bytes, passphrase, &NoKeySupport, None)
    }

    /// Opens with the passphrase and, if the vault requires one, a hardware key.
    pub fn open_with_key(bytes: &[u8], passphrase: &str, device: &dyn KeyDevice, pin: Option<&str>) -> Result<(Self, VaultData), VaultError> {
        match file_version(bytes) {
            Some(VERSION_1) => LegacyKeys::open(bytes, passphrase).map(|(keys, data)| (Self { keys: Keys::Legacy(keys) }, data)),
            Some(VERSION_2) => SlottedKeys::open(bytes, passphrase, device, pin).map(|(keys, data)| (Self { keys: Keys::Slotted(keys) }, data)),
            _ => Err(VaultError::BadFormat),
        }
    }

    /// Opens a version 2 vault with the recovery key's words instead of the passphrase.
    pub fn open_with_recovery(bytes: &[u8], normalized_phrase: &str) -> Result<(Self, VaultData), VaultError> {
        match file_version(bytes) {
            Some(VERSION_2) => SlottedKeys::open_with_recovery(bytes, normalized_phrase).map(|(keys, data)| (Self { keys: Keys::Slotted(keys) }, data)),
            Some(VERSION_1) => Err(VaultError::NeedsUpgrade),
            _ => Err(VaultError::BadFormat),
        }
    }

    fn slotted_mut(&mut self) -> Result<&mut SlottedKeys, VaultError> {
        match &mut self.keys {
            Keys::Slotted(keys) => Ok(keys),
            Keys::Legacy(_) => Err(VaultError::NeedsUpgrade),
        }
    }

    /// Replaces the passphrase without asking for the old one: for after a recovery key
    /// has already proved who is asking.
    pub fn set_passphrase(&mut self, new_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        self.slotted_mut()?.set_passphrase(new_passphrase, params)
    }

    pub fn change_passphrase(&mut self, current: &str, new_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        self.slotted_mut()?.change_passphrase(current, new_passphrase, params)
    }

    pub fn set_recovery_key(&mut self, normalized_phrase: &str, current_passphrase: &str) -> Result<(), VaultError> {
        self.slotted_mut()?.set_recovery_key(normalized_phrase, current_passphrase)
    }

    /// Follows the unlock slots of a newer copy of the vault that this session just read.
    /// Does nothing for a version 1 vault, which has none.
    pub fn adopt_slots_from(&mut self, file: &[u8]) -> Result<(), VaultError> {
        match &mut self.keys {
            Keys::Slotted(keys) => keys.adopt_slots_from(file),
            Keys::Legacy(_) => Ok(()),
        }
    }

    pub fn enable_hardware_key(&mut self, key: &KeyRegistration<'_>, current_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        self.slotted_mut()?.enable_hardware_key(key, current_passphrase, params)
    }

    pub fn add_hardware_key(&mut self, key: &KeyRegistration<'_>) -> Result<(), VaultError> {
        self.slotted_mut()?.add_hardware_key(key)
    }

    pub fn remove_hardware_key(&mut self, credential_id: &[u8]) -> Result<(), VaultError> {
        self.slotted_mut()?.remove_hardware_key(credential_id)
    }

    pub fn disable_hardware_keys(&mut self, current_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        self.slotted_mut()?.disable_hardware_keys(current_passphrase, params)
    }

    pub fn hardware_keys(&self) -> Vec<HardwareKeyInfo> {
        match &self.keys {
            Keys::Slotted(keys) => keys.hardware_keys(),
            Keys::Legacy(_) => Vec::new(),
        }
    }

    pub fn requires_hardware_key(&self) -> bool {
        matches!(&self.keys, Keys::Slotted(keys) if keys.requires_hardware_key())
    }

    pub fn has_recovery_key(&self) -> bool {
        matches!(&self.keys, Keys::Slotted(keys) if keys.has_recovery_key())
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
