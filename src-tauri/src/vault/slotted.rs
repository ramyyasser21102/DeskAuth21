//! Version 2: the data is encrypted under a random vault key, which unlock slots in
//! the header wrap. See `slot` and `layout`.

use super::crypto::{decrypt, derive_key, encrypt, random_bytes, random_key, Key, NONCE_LEN};
use super::header::SALT_LEN;
use super::layout::{header_bytes, parse};
use super::legacy::LegacyKeys;
use super::slot::{PassphraseSlot, Slot};
use super::{KdfParams, VaultError};
use crate::record::{parse_vault_data, VaultData};
use zeroize::Zeroizing;

pub struct SlottedKeys {
    vault_key: Key,
    pub(super) slots: Vec<Slot>,
}

impl SlottedKeys {
    pub fn create(passphrase: &str, params: KdfParams) -> Result<Self, VaultError> {
        let salt = random_bytes::<SALT_LEN>()?;
        let slot = PassphraseSlot::wrap(&random_key()?, &derive_key(passphrase, &salt, params)?, salt, params)?;
        // The vault key was only needed to build the slot; open it again so there is one source of truth.
        let vault_key = slot.unwrap_with_passphrase(passphrase)?;
        Ok(Self { vault_key, slots: vec![Slot::Passphrase(slot)] })
    }

    /// Moves a version 1 vault over without asking for the passphrase again: the key it
    /// already derived wraps a new vault key, with the same salt and cost, so the same
    /// passphrase keeps working.
    pub fn from_legacy(legacy: &LegacyKeys) -> Result<Self, VaultError> {
        let vault_key = random_key()?;
        let slot = PassphraseSlot::wrap(&vault_key, &legacy.key, legacy.salt, legacy.params)?;
        Ok(Self { vault_key, slots: vec![Slot::Passphrase(slot)] })
    }

    pub fn open(bytes: &[u8], passphrase: &str) -> Result<(Self, VaultData), VaultError> {
        let file = parse(bytes)?;
        let vault_key = file
            .slots
            .iter()
            .find_map(|slot| match slot {
                Slot::Passphrase(passphrase_slot) => passphrase_slot.unwrap_with_passphrase(passphrase).ok(),
                Slot::Unknown { .. } => None,
            })
            .ok_or(VaultError::Decrypt)?;
        let plaintext = decrypt(&vault_key, &file.payload_nonce, file.ciphertext, file.associated_data)?;
        Ok((Self { vault_key, slots: file.slots }, parse_vault_data(&plaintext)?))
    }

    pub fn seal(&self, data: &VaultData) -> Result<Vec<u8>, VaultError> {
        let header = header_bytes(&self.slots);
        let nonce = random_bytes::<NONCE_LEN>()?;
        let plaintext = Zeroizing::new(serde_json::to_vec(data)?);
        let ciphertext = encrypt(&self.vault_key, &nonce, &plaintext, &header)?;
        Ok([header.as_slice(), &nonce, &ciphertext].concat())
    }

    /// Decrypts another copy of this vault; its slots may differ from ours, the data key is the same.
    pub fn unseal(&self, file: &[u8]) -> Result<VaultData, VaultError> {
        let parsed = parse(file)?;
        let plaintext = decrypt(&self.vault_key, &parsed.payload_nonce, parsed.ciphertext, parsed.associated_data)?;
        Ok(parse_vault_data(&plaintext)?)
    }
}
