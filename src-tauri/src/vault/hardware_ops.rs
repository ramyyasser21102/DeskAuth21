//! Requiring a hardware key as well as the passphrase.
//!
//! With a key required there is no slot that opens the vault on its own. The passphrase
//! releases a secret (the gate), a registered key releases another (the shared secret), and
//! the envelope holds the vault key under both. See `hardware_slots`.

use super::crypto::{derive_key, random_bytes, random_key, Key};
use super::hardware_slots::{EnvelopeSlot, HardwareSlot, KeyIdentity, HMAC_SALT_LEN};
use super::header::SALT_LEN;
use super::layout::ParsedFile;
use super::passphrase_slot::{KeyCost, PassphraseSlot};
use super::slot::{Slot, KIND_PASSPHRASE_GATE};
use super::slotted::{HardwareSecrets, SlottedKeys};
use super::{KdfParams, VaultError};
use crate::key_device::{KeyDevice, KeyError, KeyRegistration};
use crate::record::VaultData;

/// What the window may show about a registered key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareKeyInfo {
    pub credential_id: Vec<u8>,
    pub label: String,
}

pub fn requires_hardware_key(slots: &[Slot]) -> bool {
    slots.iter().any(|slot| matches!(slot, Slot::Envelope(_)))
}

/// The slots that unlock with the passphrase alone or take part in the hardware key factor.
pub fn is_passphrase_or_hardware_slot(slot: &Slot) -> bool {
    matches!(slot, Slot::Passphrase(_) | Slot::PassphraseGate(_) | Slot::Hardware(_) | Slot::Envelope(_))
}

/// The first registered key that is plugged in gives the shared secret.
fn shared_secret_from_key(slots: &[Slot], device: &dyn KeyDevice, pin: Option<&str>) -> Result<Key, VaultError> {
    let mut last_error = VaultError::BadFormat;
    for slot in slots {
        let Slot::Hardware(hardware) = slot else { continue };
        match device.derive_secret(&hardware.credential_id, &hardware.salt, pin) {
            Ok(output) => return hardware.unwrap_with(&output),
            Err(KeyError::NotThisKey) => last_error = KeyError::NotThisKey.into(),
            Err(other) => return Err(other.into()),
        }
    }
    Err(last_error)
}

fn gate_slot_for(gate: &Key, passphrase: &str, params: KdfParams) -> Result<Slot, VaultError> {
    let salt = random_bytes::<SALT_LEN>()?;
    let slot = PassphraseSlot::wrap(KIND_PASSPHRASE_GATE, gate, &derive_key(passphrase, &salt, params)?, KeyCost { salt, params })?;
    Ok(Slot::PassphraseGate(slot))
}

impl SlottedKeys {
    /// The passphrase is checked first, so a wrong one never makes the person touch the key.
    pub(super) fn open_with_hardware_key(file: &ParsedFile<'_>, passphrase: &str, device: &dyn KeyDevice, pin: Option<&str>) -> Result<(Self, VaultData), VaultError> {
        let gate = file
            .slots
            .iter()
            .find_map(|slot| match slot {
                Slot::PassphraseGate(gate_slot) => gate_slot.unwrap_with_passphrase(passphrase).ok(),
                _ => None,
            })
            .ok_or(VaultError::Decrypt)?;
        let shared = shared_secret_from_key(&file.slots, device, pin)?;
        let envelope = file.slots.iter().find_map(|slot| if let Slot::Envelope(found) = slot { Some(found) } else { None }).ok_or(VaultError::BadFormat)?;
        let vault_key = envelope.unwrap_with(&gate, &shared)?;
        let parsed = ParsedFile { slots: file.slots.clone(), associated_data: file.associated_data, payload_nonce: file.payload_nonce, ciphertext: file.ciphertext };
        Self::finish_open(vault_key, parsed, Some(HardwareSecrets { gate, shared }))
    }

    /// Registers a key and makes the vault need it together with the passphrase. A recovery
    /// key must exist first, so losing the key cannot lock the owner out.
    pub fn enable_hardware_key(&mut self, key: &KeyRegistration<'_>, current_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        if requires_hardware_key(&self.slots) {
            return Err(VaultError::HardwareKeyAlreadyOn);
        }
        if !self.has_recovery_key() {
            return Err(VaultError::NeedsRecoveryKey);
        }
        self.verify_passphrase(current_passphrase)?;
        let (gate, shared) = (random_key()?, random_key()?);
        let hardware = self.register_key(key, &shared)?;
        let envelope = EnvelopeSlot::wrap(&self.vault_key, &gate, &shared)?;
        let gate_slot = gate_slot_for(&gate, current_passphrase, params)?;
        self.slots.retain(|slot| !matches!(slot, Slot::Passphrase(_)));
        self.slots.splice(0..0, [gate_slot, hardware, Slot::Envelope(envelope)]);
        self.secrets = Some(HardwareSecrets { gate, shared });
        Ok(())
    }

    fn register_key(&self, key: &KeyRegistration<'_>, shared: &Key) -> Result<Slot, VaultError> {
        let credential_id = key.device.register(key.pin)?;
        let salt = random_bytes::<HMAC_SALT_LEN>()?;
        let output = key.device.derive_secret(&credential_id, &salt, key.pin)?;
        Ok(Slot::Hardware(HardwareSlot::wrap(shared, &output, KeyIdentity { credential_id, salt, label: key.label.to_string() })?))
    }

    /// Registers another key that opens the vault just as well as the first.
    pub fn add_hardware_key(&mut self, key: &KeyRegistration<'_>) -> Result<(), VaultError> {
        if !requires_hardware_key(&self.slots) {
            return Err(VaultError::HardwareKeyNotOn);
        }
        let shared = self.secrets.as_ref().ok_or(VaultError::LockAndUnlockFirst)?.shared.clone();
        let hardware = self.register_key(key, &shared)?;
        self.slots.push(hardware);
        Ok(())
    }

    pub fn remove_hardware_key(&mut self, credential_id: &[u8]) -> Result<(), VaultError> {
        let count = self.hardware_keys().len();
        if !self.hardware_keys().iter().any(|key| key.credential_id == credential_id) {
            return Err(VaultError::UnknownHardwareKey);
        }
        if count == 1 {
            return Err(VaultError::LastHardwareKey);
        }
        self.slots.retain(|slot| !matches!(slot, Slot::Hardware(found) if found.credential_id == credential_id));
        Ok(())
    }

    /// Goes back to the passphrase alone opening the vault.
    pub fn disable_hardware_keys(&mut self, current_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        if !requires_hardware_key(&self.slots) {
            return Err(VaultError::HardwareKeyNotOn);
        }
        self.verify_passphrase(current_passphrase)?;
        self.set_passphrase(current_passphrase, params)
    }

    pub fn hardware_keys(&self) -> Vec<HardwareKeyInfo> {
        self.slots
            .iter()
            .filter_map(|slot| match slot {
                Slot::Hardware(found) => Some(HardwareKeyInfo { credential_id: found.credential_id.clone(), label: found.label.clone() }),
                _ => None,
            })
            .collect()
    }

    /// Re-wraps the gate secret under the new passphrase; the keys are not needed.
    pub(super) fn change_gate_passphrase(&mut self, new_passphrase: &str, params: KdfParams) -> Result<(), VaultError> {
        let gate = self.secrets.as_ref().ok_or(VaultError::LockAndUnlockFirst)?.gate.clone();
        let new_gate_slot = gate_slot_for(&gate, new_passphrase, params)?;
        self.slots.retain(|slot| !matches!(slot, Slot::PassphraseGate(_)));
        self.slots.insert(0, new_gate_slot);
        Ok(())
    }

    /// After taking over another computer's slots, keep the held secrets only if they still
    /// open the envelope; otherwise they belong to an older arrangement and must not be used.
    pub(super) fn drop_stale_hardware_secrets(&mut self) {
        let still_valid = match (&self.secrets, self.slots.iter().find_map(|slot| if let Slot::Envelope(found) = slot { Some(found) } else { None })) {
            (Some(secrets), Some(envelope)) => envelope.unwrap_with(&secrets.gate, &secrets.shared).is_ok_and(|key| *key == *self.vault_key),
            _ => false,
        };
        if !still_valid {
            self.secrets = None;
        }
    }

    pub fn requires_hardware_key(&self) -> bool {
        requires_hardware_key(&self.slots)
    }
}

/// Whether a vault file needs a hardware key, read from its header without any secret.
pub fn file_requires_hardware_key(bytes: &[u8]) -> bool {
    super::layout::parse(bytes).is_ok_and(|file| requires_hardware_key(&file.slots))
}
