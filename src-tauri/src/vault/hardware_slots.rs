//! The slots that make a hardware key part of the key to the vault.
//!
//! Unlocking then needs two things at once. The passphrase releases a secret `gate`; a
//! hardware key releases a secret `shared`; an envelope holds the vault key under a key made
//! from both. Each registered key holds its own wrapping of the same `shared` secret, and the
//! passphrase part is a separate slot, so adding a key or changing the passphrase never needs
//! the other keys.

use super::crypto::{decrypt, encrypt, hmac_sha256, random_bytes, Key, KEY_LEN, NONCE_LEN};
use super::VaultError;
use zeroize::Zeroizing;

pub const KIND_HARDWARE: u8 = 3;
pub const KIND_ENVELOPE: u8 = 5;
pub const HMAC_SALT_LEN: usize = 32;
pub const MAX_LABEL_BYTES: usize = 32;
const WRAPPED_LEN: usize = KEY_LEN + 16;
const U16_LEN: usize = 2;
const AAD_PREFIX: &[u8] = b"DA21 slot";
const HARDWARE_CONTEXT: &[u8] = b"DA21 hardware key v1";
const ENVELOPE_CONTEXT: &[u8] = b"DA21 envelope v1";

/// What names one registered key: its credential, the salt it derives from, and its label.
pub struct KeyIdentity {
    pub credential_id: Vec<u8>,
    pub salt: [u8; HMAC_SALT_LEN],
    pub label: String,
}

/// One registered key: which credential on it, the salt it derives from, and the shared
/// secret wrapped under what the key produces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareSlot {
    pub credential_id: Vec<u8>,
    pub salt: [u8; HMAC_SALT_LEN],
    pub label: String,
    nonce: [u8; NONCE_LEN],
    wrapped_secret: [u8; WRAPPED_LEN],
}

/// The vault key, under a key that needs both the passphrase part and the hardware part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeSlot {
    nonce: [u8; NONCE_LEN],
    wrapped_key: [u8; WRAPPED_LEN],
}

fn hardware_aad(credential_id: &[u8], salt: &[u8], nonce: &[u8]) -> Vec<u8> {
    [AAD_PREFIX, &[KIND_HARDWARE], credential_id, salt, nonce].concat()
}

fn envelope_aad(nonce: &[u8]) -> Vec<u8> {
    [AAD_PREFIX, &[KIND_ENVELOPE], nonce].concat()
}

fn key_from(plain: &[u8]) -> Result<Key, VaultError> {
    let bytes: [u8; KEY_LEN] = plain.try_into().map_err(|_| VaultError::Decrypt)?;
    Ok(Zeroizing::new(bytes))
}

impl HardwareSlot {
    /// Wraps `shared_secret` under what a key derived from this credential and salt.
    pub fn wrap(shared_secret: &Key, key_output: &[u8; 32], identity: KeyIdentity) -> Result<Self, VaultError> {
        let KeyIdentity { credential_id, salt, label } = identity;
        let nonce = random_bytes::<NONCE_LEN>()?;
        let kek = hmac_sha256(HARDWARE_CONTEXT, key_output);
        let wrapped = encrypt(&kek, &nonce, shared_secret.as_ref(), &hardware_aad(&credential_id, &salt, &nonce))?;
        let label = truncate_label(&label);
        Ok(Self { credential_id, salt, label, nonce, wrapped_secret: wrapped.try_into().map_err(|_| VaultError::Encrypt)? })
    }

    pub fn unwrap_with(&self, key_output: &[u8; 32]) -> Result<Key, VaultError> {
        let kek = hmac_sha256(HARDWARE_CONTEXT, key_output);
        let aad = hardware_aad(&self.credential_id, &self.salt, &self.nonce);
        key_from(&decrypt(&kek, &self.nonce, &self.wrapped_secret, &aad)?)
    }

    pub fn encode_body(&self) -> Vec<u8> {
        let id_len = u16::try_from(self.credential_id.len()).expect("credential ids are far below 64 KiB");
        let label = self.label.as_bytes();
        let label_len = u8::try_from(label.len()).expect("labels are cut to MAX_LABEL_BYTES");
        [&id_len.to_be_bytes()[..], &self.credential_id, &self.salt, &self.nonce, &self.wrapped_secret, &[label_len], label].concat()
    }

    pub fn decode_body(body: &[u8]) -> Result<Self, VaultError> {
        let id_len = usize::from(u16::from_be_bytes(body.get(..U16_LEN).ok_or(VaultError::BadFormat)?.try_into().map_err(|_| VaultError::BadFormat)?));
        let mut reader = Reader { rest: &body[U16_LEN..] };
        let credential_id = reader.take(id_len)?.to_vec();
        let salt = reader.take_array::<HMAC_SALT_LEN>()?;
        let nonce = reader.take_array::<NONCE_LEN>()?;
        let wrapped_secret = reader.take_array::<WRAPPED_LEN>()?;
        let label_len = usize::from(reader.take(1)?[0]);
        let label = String::from_utf8(reader.take(label_len)?.to_vec()).map_err(|_| VaultError::BadFormat)?;
        if !reader.rest.is_empty() || label.len() > MAX_LABEL_BYTES {
            return Err(VaultError::BadFormat);
        }
        Ok(Self { credential_id, salt, label, nonce, wrapped_secret })
    }
}

impl EnvelopeSlot {
    pub fn wrap(vault_key: &Key, gate: &Key, shared_secret: &Key) -> Result<Self, VaultError> {
        let nonce = random_bytes::<NONCE_LEN>()?;
        let wrapped = encrypt(&envelope_key(gate, shared_secret), &nonce, vault_key.as_ref(), &envelope_aad(&nonce))?;
        Ok(Self { nonce, wrapped_key: wrapped.try_into().map_err(|_| VaultError::Encrypt)? })
    }

    pub fn unwrap_with(&self, gate: &Key, shared_secret: &Key) -> Result<Key, VaultError> {
        key_from(&decrypt(&envelope_key(gate, shared_secret), &self.nonce, &self.wrapped_key, &envelope_aad(&self.nonce))?)
    }

    pub fn encode_body(&self) -> Vec<u8> {
        [&self.nonce[..], &self.wrapped_key].concat()
    }

    pub fn decode_body(body: &[u8]) -> Result<Self, VaultError> {
        let mut reader = Reader { rest: body };
        let nonce = reader.take_array::<NONCE_LEN>()?;
        let wrapped_key = reader.take_array::<WRAPPED_LEN>()?;
        if reader.rest.is_empty() { Ok(Self { nonce, wrapped_key }) } else { Err(VaultError::BadFormat) }
    }
}

fn envelope_key(gate: &Key, shared_secret: &Key) -> Key {
    hmac_sha256(gate.as_ref(), &[shared_secret.as_ref(), ENVELOPE_CONTEXT].concat())
}

/// Cuts a label to `MAX_LABEL_BYTES` without splitting a character.
pub fn truncate_label(label: &str) -> String {
    let mut end = label.len().min(MAX_LABEL_BYTES);
    while !label.is_char_boundary(end) {
        end -= 1;
    }
    label[..end].to_string()
}

struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], VaultError> {
        let (taken, rest) = self.rest.split_at_checked(count).ok_or(VaultError::BadFormat)?;
        self.rest = rest;
        Ok(taken)
    }

    fn take_array<const LEN: usize>(&mut self) -> Result<[u8; LEN], VaultError> {
        self.take(LEN)?.try_into().map_err(|_| VaultError::BadFormat)
    }
}
