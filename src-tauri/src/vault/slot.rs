//! An unlock slot: one way of recovering the vault key, kept in the file header.
//!
//! The vault's data is encrypted under a random vault key. Each slot holds a secret wrapped
//! (encrypted) under a key made from something the person has: the passphrase, the recovery
//! key, or (when one is required) a hardware key. Adding or changing a way to unlock rewrites
//! slots, never the data key.

use super::crypto::{decrypt, encrypt, random_bytes, Key, KEY_LEN, NONCE_LEN};
use super::hardware_slots::{EnvelopeSlot, HardwareSlot, KIND_ENVELOPE, KIND_HARDWARE};
use super::passphrase_slot::PassphraseSlot;
use super::VaultError;
use zeroize::Zeroizing;

pub const KIND_PASSPHRASE: u8 = 1;
pub const KIND_RECOVERY: u8 = 2;
/// The passphrase half of the key to a vault that also needs a hardware key.
pub const KIND_PASSPHRASE_GATE: u8 = 4;
const WRAPPED_LEN: usize = KEY_LEN + 16;
const RECOVERY_BODY_LEN: usize = NONCE_LEN + WRAPPED_LEN;
const SLOT_AAD_PREFIX: &[u8] = b"DA21 slot";

/// Opens the vault with the recovery key's words instead of the passphrase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySlot {
    nonce: [u8; NONCE_LEN],
    wrapped_key: [u8; WRAPPED_LEN],
}

/// `Unknown` keeps a kind this version does not understand exactly as it was read,
/// so saving never drops a way of unlocking that a newer version added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slot {
    Passphrase(PassphraseSlot),
    PassphraseGate(PassphraseSlot),
    Recovery(RecoverySlot),
    Hardware(HardwareSlot),
    Envelope(EnvelopeSlot),
    Unknown { kind: u8, body: Vec<u8> },
}

impl RecoverySlot {
    pub fn wrap(vault_key: &Key, key_encryption_key: &Key) -> Result<Self, VaultError> {
        let nonce = random_bytes::<NONCE_LEN>()?;
        let wrapped = encrypt(key_encryption_key, &nonce, vault_key.as_ref(), &recovery_associated_data(&nonce))?;
        Ok(Self { nonce, wrapped_key: wrapped.try_into().map_err(|_| VaultError::Encrypt)? })
    }

    pub fn unwrap_with(&self, key_encryption_key: &Key) -> Result<Key, VaultError> {
        let plain = decrypt(key_encryption_key, &self.nonce, &self.wrapped_key, &recovery_associated_data(&self.nonce))?;
        let bytes: [u8; KEY_LEN] = plain.as_slice().try_into().map_err(|_| VaultError::Decrypt)?;
        Ok(Zeroizing::new(bytes))
    }
}

fn recovery_associated_data(nonce: &[u8]) -> Vec<u8> {
    [SLOT_AAD_PREFIX, &[KIND_RECOVERY], nonce].concat()
}

impl Slot {
    pub fn kind(&self) -> u8 {
        match self {
            Self::Passphrase(_) => KIND_PASSPHRASE,
            Self::PassphraseGate(_) => KIND_PASSPHRASE_GATE,
            Self::Recovery(_) => KIND_RECOVERY,
            Self::Hardware(_) => KIND_HARDWARE,
            Self::Envelope(_) => KIND_ENVELOPE,
            Self::Unknown { kind, .. } => *kind,
        }
    }

    pub fn encode_body(&self) -> Vec<u8> {
        match self {
            Self::Passphrase(slot) | Self::PassphraseGate(slot) => slot.encode_body(),
            Self::Recovery(slot) => [&slot.nonce[..], &slot.wrapped_key].concat(),
            Self::Hardware(slot) => slot.encode_body(),
            Self::Envelope(slot) => slot.encode_body(),
            Self::Unknown { body, .. } => body.clone(),
        }
    }

    pub fn decode(kind: u8, body: &[u8]) -> Result<Self, VaultError> {
        match kind {
            KIND_PASSPHRASE => Ok(Self::Passphrase(PassphraseSlot::decode_body(kind, body)?)),
            KIND_PASSPHRASE_GATE => Ok(Self::PassphraseGate(PassphraseSlot::decode_body(kind, body)?)),
            KIND_RECOVERY => Self::decode_recovery(body),
            KIND_HARDWARE => Ok(Self::Hardware(HardwareSlot::decode_body(body)?)),
            KIND_ENVELOPE => Ok(Self::Envelope(EnvelopeSlot::decode_body(body)?)),
            _ => Ok(Self::Unknown { kind, body: body.to_vec() }),
        }
    }

    fn decode_recovery(body: &[u8]) -> Result<Self, VaultError> {
        if body.len() != RECOVERY_BODY_LEN {
            return Err(VaultError::BadFormat);
        }
        let (nonce, wrapped_key) = body.split_at(NONCE_LEN);
        Ok(Self::Recovery(RecoverySlot {
            nonce: nonce.try_into().map_err(|_| VaultError::BadFormat)?,
            wrapped_key: wrapped_key.try_into().map_err(|_| VaultError::BadFormat)?,
        }))
    }
}
