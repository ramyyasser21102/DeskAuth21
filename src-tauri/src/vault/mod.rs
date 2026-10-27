//! Encrypted on-disk vault: Argon2id key derivation + XChaCha20-Poly1305.
//!
//! The file is `header || ciphertext`; see [`header`] for the header layout.
//! The plaintext is the JSON-serialised account list.

mod header;
#[cfg(test)]
mod tests;

use crate::otp::Account;
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::XChaCha20Poly1305;
use header::{associated_data, Header, HEADER_LEN, NONCE_LEN, SALT_LEN};
use zeroize::Zeroizing;

const KEY_LEN: usize = 32;
type Key = Zeroizing<[u8; KEY_LEN]>;

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("not a DeskAuth21 vault, or an unsupported version")]
    BadFormat,
    #[error("wrong passphrase or corrupted vault")]
    Decrypt,
    #[error("could not encrypt the vault")]
    Encrypt,
    #[error("invalid key derivation parameters")]
    BadParams,
    #[error("could not gather randomness from the OS")]
    Random,
    #[error("vault contents are not valid: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub memory_kib: u32,
    pub passes: u32,
    pub lanes: u32,
}

impl KdfParams {
    /// 64 MiB, 3 passes, 1 lane: a sensible interactive default for desktops.
    pub const DEFAULT: Self = Self { memory_kib: 64 * 1024, passes: 3, lanes: 1 };
}

/// An unlocked vault: the derived key plus what is needed to re-seal quickly,
/// so the slow KDF runs once per unlock rather than once per save.
pub struct Session {
    key: Key,
    salt: [u8; SALT_LEN],
    params: KdfParams,
}

impl Session {
    /// Start a brand-new vault with a fresh random salt.
    pub fn create(passphrase: &str, params: KdfParams) -> Result<Self, VaultError> {
        let salt = random_bytes::<SALT_LEN>()?;
        Ok(Self { key: derive_key(passphrase, &salt, params)?, salt, params })
    }

    /// Encrypt the account list under a fresh random nonce.
    pub fn seal(&self, accounts: &[Account]) -> Result<Vec<u8>, VaultError> {
        let header = Header { params: self.params, salt: self.salt, nonce: random_bytes()? };
        let encoded_header = header.encode();
        let plaintext = Zeroizing::new(serde_json::to_vec(accounts)?);
        let ciphertext = build_cipher(&self.key)?
            .encrypt(
                (&header.nonce).into(),
                Payload { msg: &plaintext, aad: associated_data(&encoded_header) },
            )
            .map_err(|_| VaultError::Encrypt)?;
        Ok([encoded_header.as_slice(), &ciphertext].concat())
    }

    /// Decrypt another copy of this vault with the key already held, without
    /// re-deriving it. Fails if the file was made with a different key.
    pub fn unseal(&self, file: &[u8]) -> Result<Vec<Account>, VaultError> {
        let header = Header::decode(file)?;
        let plaintext = decrypt_payload(&self.key, &header, file)?;
        Ok(serde_json::from_slice(&plaintext)?)
    }

    /// Decrypt a vault file, returning the live session and its accounts.
    pub fn open(bytes: &[u8], passphrase: &str) -> Result<(Self, Vec<Account>), VaultError> {
        let header = Header::decode(bytes)?;
        let key = derive_key(passphrase, &header.salt, header.params)?;
        let plaintext = decrypt_payload(&key, &header, bytes)?;
        let accounts = serde_json::from_slice(&plaintext)?;
        Ok((Self { key, salt: header.salt, params: header.params }, accounts))
    }
}

fn decrypt_payload(key: &Key, header: &Header, file: &[u8]) -> Result<Zeroizing<Vec<u8>>, VaultError> {
    let payload = Payload { msg: &file[HEADER_LEN..], aad: associated_data(file) };
    build_cipher(key)?
        .decrypt((&header.nonce).into(), payload)
        .map(Zeroizing::new)
        .map_err(|_| VaultError::Decrypt)
}

fn build_cipher(key: &Key) -> Result<XChaCha20Poly1305, VaultError> {
    XChaCha20Poly1305::new_from_slice(key.as_ref()).map_err(|_| VaultError::BadParams)
}

fn derive_key(passphrase: &str, salt: &[u8], params: KdfParams) -> Result<Key, VaultError> {
    let argon_params = Params::new(params.memory_kib, params.passes, params.lanes, Some(KEY_LEN))
        .map_err(|_| VaultError::BadParams)?;
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params)
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|_| VaultError::BadParams)?;
    Ok(key)
}

fn random_bytes<const LEN: usize>() -> Result<[u8; LEN], VaultError> {
    let mut bytes = [0u8; LEN];
    getrandom::fill(&mut bytes).map_err(|_| VaultError::Random)?;
    Ok(bytes)
}

const _: () = assert!(NONCE_LEN == 24, "XChaCha20 requires a 192-bit nonce");
