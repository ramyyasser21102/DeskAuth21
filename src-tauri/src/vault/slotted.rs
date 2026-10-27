//! Version 2: the data is encrypted under a random vault key, which unlock slots in
//! the header wrap. See `slot` and `layout`.

use super::crypto::{decrypt, derive_key, encrypt, random_bytes, random_key, recovery_key_encryption_key, Key, NONCE_LEN};
use super::header::SALT_LEN;
use super::layout::{header_bytes, parse};
use super::legacy::LegacyKeys;
use super::slot::{PassphraseSlot, RecoverySlot, Slot};
use super::{KdfParams, VaultError};
use crate::record::{parse_vault_data, VaultData};
use zeroize::Zeroizing;

#[derive(Clone)]
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
                Slot::Recovery(_) | Slot::Unknown { .. } => None,
            })
            .ok_or(VaultError::Decrypt)?;
        let plaintext = decrypt(&vault_key, &file.payload_nonce, file.ciphertext, file.associated_data)?;
        Ok((Self { vault_key, slots: file.slots }, parse_vault_data(&plaintext)?))
    }

    /// Opens the vault with the recovery key's words (already put in canonical form).
    pub fn open_with_recovery(bytes: &[u8], normalized_phrase: &str) -> Result<(Self, VaultData), VaultError> {
        let file = parse(bytes)?;
        let key_encryption_key = recovery_key_encryption_key(normalized_phrase);
        let vault_key = file
            .slots
            .iter()
            .find_map(|slot| match slot {
                Slot::Recovery(recovery_slot) => recovery_slot.unwrap_with(&key_encryption_key).ok(),
                Slot::Passphrase(_) | Slot::Unknown { .. } => None,
            })
            .ok_or(VaultError::WrongRecoveryKey)?;
        let plaintext = decrypt(&vault_key, &file.payload_nonce, file.ciphertext, file.associated_data)?;
        Ok((Self { vault_key, slots: file.slots }, parse_vault_data(&plaintext)?))
    }

    /// Proves the person knows the current passphrase, for changes that must not be
    /// possible for someone sitting at an already unlocked computer.
    pub fn verify_passphrase(&self, passphrase: &str) -> Result<(), VaultError> {
        let matches_a_slot = self.slots.iter().any(|slot| match slot {
            Slot::Passphrase(passphrase_slot) => passphrase_slot.unwrap_with_passphrase(passphrase).is_ok(),
            Slot::Recovery(_) | Slot::Unknown { .. } => false,
        });
        if matches_a_slot { Ok(()) } else { Err(VaultError::Decrypt) }
    }

    /// Replaces the passphrase slot with one made from `new_passphrase`, with a fresh salt.
    pub fn set_passphrase(&mut self, new_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        let salt = random_bytes::<SALT_LEN>()?;
        let slot = PassphraseSlot::wrap(&self.vault_key, &derive_key(new_passphrase, &salt, params)?, salt, params)?;
        self.slots.retain(|existing| !matches!(existing, Slot::Passphrase(_)));
        self.slots.insert(0, Slot::Passphrase(slot));
        Ok(())
    }

    pub fn change_passphrase(&mut self, current: &str, new_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        self.verify_passphrase(current)?;
        self.set_passphrase(new_passphrase, params)
    }

    /// Adds, or replaces, the recovery slot for these words, after checking the passphrase.
    pub fn set_recovery_key(&mut self, normalized_phrase: &str, current_passphrase: &str) -> Result<(), VaultError> {
        self.verify_passphrase(current_passphrase)?;
        let slot = RecoverySlot::wrap(&self.vault_key, &recovery_key_encryption_key(normalized_phrase))?;
        self.slots.retain(|existing| !matches!(existing, Slot::Recovery(_)));
        self.slots.push(Slot::Recovery(slot));
        Ok(())
    }

    /// Takes over the unlock slots of another copy of this vault that opened with our
    /// vault key, so a passphrase or recovery key changed on another computer is not undone
    /// the next time this computer saves.
    pub fn adopt_slots_from(&mut self, file: &[u8]) -> Result<(), VaultError> {
        self.slots = parse(file)?.slots;
        Ok(())
    }

    pub fn has_recovery_key(&self) -> bool {
        self.slots.iter().any(|slot| matches!(slot, Slot::Recovery(_)))
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
