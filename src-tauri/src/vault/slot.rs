//! An unlock slot: one way of recovering the vault key, kept in the file header.
//!
//! The vault's data is encrypted under a random vault key. Each slot holds that key
//! wrapped (encrypted) under a key made from something the person has: today the
//! passphrase. Adding or changing a way to unlock rewrites slots, never the data key.

use super::crypto::{decrypt, derive_key, encrypt, random_bytes, Key, KEY_LEN, NONCE_LEN};
use super::header::{ensure_within_limits, SALT_LEN};
use super::{KdfParams, VaultError};
use zeroize::Zeroizing;

pub const KIND_PASSPHRASE: u8 = 1;
pub const KIND_RECOVERY: u8 = 2;
const TAG_LEN: usize = 16;
const WRAPPED_LEN: usize = KEY_LEN + TAG_LEN;
const U32_LEN: usize = 4;
const PARAMS_LEN: usize = 3 * U32_LEN;
const PASSPHRASE_BODY_LEN: usize = PARAMS_LEN + SALT_LEN + NONCE_LEN + WRAPPED_LEN;
const RECOVERY_BODY_LEN: usize = NONCE_LEN + WRAPPED_LEN;
const SLOT_AAD_PREFIX: &[u8] = b"DA21 slot";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassphraseSlot {
    pub params: KdfParams,
    pub salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
    wrapped_key: [u8; WRAPPED_LEN],
}

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
    Recovery(RecoverySlot),
    Unknown { kind: u8, body: Vec<u8> },
}

impl PassphraseSlot {
    /// Wraps `vault_key` under `key_encryption_key`, which came from the passphrase
    /// with exactly this salt and cost.
    pub fn wrap(vault_key: &Key, key_encryption_key: &Key, salt: [u8; SALT_LEN], params: KdfParams) -> Result<Self, VaultError> {
        let nonce = random_bytes::<NONCE_LEN>()?;
        let aad = associated_data(&params, &salt, &nonce);
        let wrapped = encrypt(key_encryption_key, &nonce, vault_key.as_ref(), &aad)?;
        let wrapped_key: [u8; WRAPPED_LEN] = wrapped.try_into().map_err(|_| VaultError::Encrypt)?;
        Ok(Self { params, salt, nonce, wrapped_key })
    }

    pub fn unwrap_with(&self, key_encryption_key: &Key) -> Result<Key, VaultError> {
        let aad = associated_data(&self.params, &self.salt, &self.nonce);
        let plain = decrypt(key_encryption_key, &self.nonce, &self.wrapped_key, &aad)?;
        let bytes: [u8; KEY_LEN] = plain.as_slice().try_into().map_err(|_| VaultError::Decrypt)?;
        Ok(Zeroizing::new(bytes))
    }

    /// Slow by design: this is where the passphrase is turned into a key.
    pub fn unwrap_with_passphrase(&self, passphrase: &str) -> Result<Key, VaultError> {
        self.unwrap_with(&derive_key(passphrase, &self.salt, self.params)?)
    }
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

/// Binds the cost, salt and nonce to the wrapped key, so editing any of them fails.
fn associated_data(params: &KdfParams, salt: &[u8], nonce: &[u8]) -> Vec<u8> {
    [SLOT_AAD_PREFIX, &[KIND_PASSPHRASE], &encode_params(params), salt, nonce].concat()
}

fn encode_params(params: &KdfParams) -> [u8; PARAMS_LEN] {
    let mut bytes = [0u8; PARAMS_LEN];
    for (chunk, value) in bytes.chunks_mut(U32_LEN).zip([params.memory_kib, params.passes, params.lanes]) {
        chunk.copy_from_slice(&value.to_be_bytes());
    }
    bytes
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes[..U32_LEN].try_into().expect("a u32 is four bytes"))
}

impl Slot {
    pub fn kind(&self) -> u8 {
        match self {
            Self::Passphrase(_) => KIND_PASSPHRASE,
            Self::Recovery(_) => KIND_RECOVERY,
            Self::Unknown { kind, .. } => *kind,
        }
    }

    pub fn encode_body(&self) -> Vec<u8> {
        match self {
            Self::Passphrase(slot) => [&encode_params(&slot.params)[..], &slot.salt, &slot.nonce, &slot.wrapped_key].concat(),
            Self::Recovery(slot) => [&slot.nonce[..], &slot.wrapped_key].concat(),
            Self::Unknown { body, .. } => body.clone(),
        }
    }

    pub fn decode(kind: u8, body: &[u8]) -> Result<Self, VaultError> {
        match kind {
            KIND_PASSPHRASE => Self::decode_passphrase(body),
            KIND_RECOVERY => Self::decode_recovery(body),
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

    fn decode_passphrase(body: &[u8]) -> Result<Self, VaultError> {
        if body.len() != PASSPHRASE_BODY_LEN {
            return Err(VaultError::BadFormat);
        }
        let params = KdfParams { memory_kib: read_u32(body), passes: read_u32(&body[U32_LEN..]), lanes: read_u32(&body[2 * U32_LEN..]) };
        ensure_within_limits(params)?;
        let rest = &body[PARAMS_LEN..];
        let (salt, rest) = rest.split_at(SALT_LEN);
        let (nonce, wrapped_key) = rest.split_at(NONCE_LEN);
        Ok(Self::Passphrase(PassphraseSlot {
            params,
            salt: salt.try_into().map_err(|_| VaultError::BadFormat)?,
            nonce: nonce.try_into().map_err(|_| VaultError::BadFormat)?,
            wrapped_key: wrapped_key.try_into().map_err(|_| VaultError::BadFormat)?,
        }))
    }
}
