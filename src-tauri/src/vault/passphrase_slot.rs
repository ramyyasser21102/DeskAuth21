//! A slot whose secret is wrapped under a key made from the passphrase. Used both for the
//! vault key itself (kind 1) and, once a hardware key is required, for the passphrase half
//! of the key to the vault (kind 4, the "gate").

use super::crypto::{decrypt, derive_key, encrypt, random_bytes, Key, KEY_LEN, NONCE_LEN};
use super::header::{ensure_within_limits, SALT_LEN};
use super::{KdfParams, VaultError};
use zeroize::Zeroizing;

const TAG_LEN: usize = 16;
const WRAPPED_LEN: usize = KEY_LEN + TAG_LEN;
const U32_LEN: usize = 4;
const PARAMS_LEN: usize = 3 * U32_LEN;
pub const PASSPHRASE_BODY_LEN: usize = PARAMS_LEN + SALT_LEN + NONCE_LEN + WRAPPED_LEN;
const SLOT_AAD_PREFIX: &[u8] = b"DA21 slot";

/// The salt and cost a passphrase is turned into a key with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCost {
    pub salt: [u8; SALT_LEN],
    pub params: KdfParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassphraseSlot {
    pub kind: u8,
    pub params: KdfParams,
    pub salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
    wrapped_key: [u8; WRAPPED_LEN],
}

impl PassphraseSlot {
    /// Wraps `secret` under `key_encryption_key`, which came from the passphrase with
    /// exactly this salt and cost.
    pub fn wrap(kind: u8, secret: &Key, key_encryption_key: &Key, cost: KeyCost) -> Result<Self, VaultError> {
        let KeyCost { salt, params } = cost;
        let nonce = random_bytes::<NONCE_LEN>()?;
        let aad = associated_data(kind, &params, &salt, &nonce);
        let wrapped = encrypt(key_encryption_key, &nonce, secret.as_ref(), &aad)?;
        let wrapped_key: [u8; WRAPPED_LEN] = wrapped.try_into().map_err(|_| VaultError::Encrypt)?;
        Ok(Self { kind, params, salt, nonce, wrapped_key })
    }

    pub fn unwrap_with(&self, key_encryption_key: &Key) -> Result<Key, VaultError> {
        let aad = associated_data(self.kind, &self.params, &self.salt, &self.nonce);
        let plain = decrypt(key_encryption_key, &self.nonce, &self.wrapped_key, &aad)?;
        let bytes: [u8; KEY_LEN] = plain.as_slice().try_into().map_err(|_| VaultError::Decrypt)?;
        Ok(Zeroizing::new(bytes))
    }

    /// Slow by design: this is where the passphrase is turned into a key.
    pub fn unwrap_with_passphrase(&self, passphrase: &str) -> Result<Key, VaultError> {
        self.unwrap_with(&derive_key(passphrase, &self.salt, self.params)?)
    }

    pub fn encode_body(&self) -> Vec<u8> {
        [&encode_params(&self.params)[..], &self.salt, &self.nonce, &self.wrapped_key].concat()
    }

    pub fn decode_body(kind: u8, body: &[u8]) -> Result<Self, VaultError> {
        if body.len() != PASSPHRASE_BODY_LEN {
            return Err(VaultError::BadFormat);
        }
        let params = KdfParams { memory_kib: read_u32(body), passes: read_u32(&body[U32_LEN..]), lanes: read_u32(&body[2 * U32_LEN..]) };
        ensure_within_limits(params)?;
        let rest = &body[PARAMS_LEN..];
        let (salt, rest) = rest.split_at(SALT_LEN);
        let (nonce, wrapped_key) = rest.split_at(NONCE_LEN);
        Ok(Self {
            kind,
            params,
            salt: salt.try_into().map_err(|_| VaultError::BadFormat)?,
            nonce: nonce.try_into().map_err(|_| VaultError::BadFormat)?,
            wrapped_key: wrapped_key.try_into().map_err(|_| VaultError::BadFormat)?,
        })
    }
}

/// Binds the kind, cost, salt and nonce to the wrapped key, so editing any of them fails.
fn associated_data(kind: u8, params: &KdfParams, salt: &[u8], nonce: &[u8]) -> Vec<u8> {
    [SLOT_AAD_PREFIX, &[kind], &encode_params(params), salt, nonce].concat()
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
