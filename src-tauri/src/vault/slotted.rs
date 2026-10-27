//! Version 2: the data is encrypted under a random vault key, which unlock slots in
//! the header wrap. See `slot`, `layout` and, for hardware keys, `hardware_ops`.

use super::crypto::{decrypt, derive_key, encrypt, random_bytes, random_key, recovery_key_encryption_key, Key, NONCE_LEN};
use super::header::SALT_LEN;
use super::layout::{header_bytes, parse};
use super::legacy::LegacyKeys;
use super::passphrase_slot::{KeyCost, PassphraseSlot};
use super::slot::{RecoverySlot, Slot, KIND_PASSPHRASE};
use super::{KdfParams, VaultError};
use crate::key_device::KeyDevice;
use crate::record::{parse_vault_data, VaultData};
use zeroize::Zeroizing;

/// What a session that opened a hardware-key vault holds, so it can add keys and change the
/// passphrase without touching the other keys. Gone when the vault locks.
#[derive(Clone)]
pub struct HardwareSecrets {
    pub(super) gate: Key,
    pub(super) shared: Key,
}

#[derive(Clone)]
pub struct SlottedKeys {
    pub(super) vault_key: Key,
    pub(super) slots: Vec<Slot>,
    pub(super) secrets: Option<HardwareSecrets>,
}

impl SlottedKeys {
    pub fn create(passphrase: &str, params: KdfParams) -> Result<Self, VaultError> {
        let vault_key = random_key()?;
        let mut keys = Self { vault_key, slots: Vec::new(), secrets: None };
        keys.set_passphrase(passphrase, params)?;
        Ok(keys)
    }

    /// Moves a version 1 vault over without asking for the passphrase again: the key it
    /// already derived wraps a new vault key, with the same salt and cost, so the same
    /// passphrase keeps working.
    pub fn from_legacy(legacy: &LegacyKeys) -> Result<Self, VaultError> {
        let vault_key = random_key()?;
        let slot = PassphraseSlot::wrap(KIND_PASSPHRASE, &vault_key, &legacy.key, KeyCost { salt: legacy.salt, params: legacy.params })?;
        Ok(Self { vault_key, slots: vec![Slot::Passphrase(slot)], secrets: None })
    }

    /// Opens with the passphrase, and the hardware key too when the vault requires one.
    pub fn open(bytes: &[u8], passphrase: &str, device: &dyn KeyDevice, pin: Option<&str>) -> Result<(Self, VaultData), VaultError> {
        let file = parse(bytes)?;
        if super::hardware_ops::requires_hardware_key(&file.slots) {
            return Self::open_with_hardware_key(&file, passphrase, device, pin);
        }
        let vault_key = file
            .slots
            .iter()
            .find_map(|slot| match slot {
                Slot::Passphrase(passphrase_slot) => passphrase_slot.unwrap_with_passphrase(passphrase).ok(),
                _ => None,
            })
            .ok_or(VaultError::Decrypt)?;
        Self::finish_open(vault_key, file, None)
    }

    pub(super) fn finish_open(vault_key: Key, file: super::layout::ParsedFile<'_>, secrets: Option<HardwareSecrets>) -> Result<(Self, VaultData), VaultError> {
        let plaintext = decrypt(&vault_key, &file.payload_nonce, file.ciphertext, file.associated_data)?;
        Ok((Self { vault_key, slots: file.slots, secrets }, parse_vault_data(&plaintext)?))
    }

    /// Opens with the recovery key's words (already in canonical form). The hardware
    /// secrets are not known afterwards, so a hardware key requirement is dropped when the
    /// passphrase is reset (see `set_passphrase`).
    pub fn open_with_recovery(bytes: &[u8], normalized_phrase: &str) -> Result<(Self, VaultData), VaultError> {
        let file = parse(bytes)?;
        let key_encryption_key = recovery_key_encryption_key(normalized_phrase);
        let vault_key = file
            .slots
            .iter()
            .find_map(|slot| match slot {
                Slot::Recovery(recovery_slot) => recovery_slot.unwrap_with(&key_encryption_key).ok(),
                _ => None,
            })
            .ok_or(VaultError::WrongRecoveryKey)?;
        Self::finish_open(vault_key, file, None)
    }

    /// Proves the person knows the current passphrase, for changes that must not be
    /// possible for someone sitting at an already unlocked computer.
    pub fn verify_passphrase(&self, passphrase: &str) -> Result<(), VaultError> {
        let matches_a_slot = self.slots.iter().any(|slot| match slot {
            Slot::Passphrase(passphrase_slot) | Slot::PassphraseGate(passphrase_slot) => passphrase_slot.unwrap_with_passphrase(passphrase).is_ok(),
            _ => false,
        });
        if matches_a_slot { Ok(()) } else { Err(VaultError::Decrypt) }
    }

    /// Makes the passphrase alone open the vault, with a fresh salt. Any hardware key
    /// requirement is removed: use it for a vault whose passphrase was lost.
    pub fn set_passphrase(&mut self, new_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        let salt = random_bytes::<SALT_LEN>()?;
        let slot = PassphraseSlot::wrap(KIND_PASSPHRASE, &self.vault_key, &derive_key(new_passphrase, &salt, params)?, KeyCost { salt, params })?;
        self.slots.retain(|existing| !super::hardware_ops::is_passphrase_or_hardware_slot(existing));
        self.slots.insert(0, Slot::Passphrase(slot));
        self.secrets = None;
        Ok(())
    }

    pub fn change_passphrase(&mut self, current: &str, new_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        self.verify_passphrase(current)?;
        if super::hardware_ops::requires_hardware_key(&self.slots) {
            return self.change_gate_passphrase(new_passphrase, params);
        }
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

    pub fn has_recovery_key(&self) -> bool {
        self.slots.iter().any(|slot| matches!(slot, Slot::Recovery(_)))
    }

    /// Takes over the unlock slots of another copy of this vault that opened with our
    /// vault key, so a change made on another computer is not undone the next time this
    /// computer saves. Hardware secrets held from an earlier unlock are dropped if the
    /// adopted slots no longer match them.
    pub fn adopt_slots_from(&mut self, file: &[u8]) -> Result<(), VaultError> {
        self.slots = parse(file)?.slots;
        self.drop_stale_hardware_secrets();
        Ok(())
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
