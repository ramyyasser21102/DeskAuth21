//! The primitives the vault formats share: key derivation, AEAD and randomness.

use super::{KdfParams, VaultError};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::XChaCha20Poly1305;
use zeroize::Zeroizing;

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;
pub type Key = Zeroizing<[u8; KEY_LEN]>;

fn build_cipher(key: &Key) -> Result<XChaCha20Poly1305, VaultError> {
    XChaCha20Poly1305::new_from_slice(key.as_ref()).map_err(|_| VaultError::BadParams)
}

pub fn derive_key(passphrase: &str, salt: &[u8], params: KdfParams) -> Result<Key, VaultError> {
    let argon_params = Params::new(params.memory_kib, params.passes, params.lanes, Some(KEY_LEN))
        .map_err(|_| VaultError::BadParams)?;
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params)
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|_| VaultError::BadParams)?;
    Ok(key)
}

pub fn random_bytes<const LEN: usize>() -> Result<[u8; LEN], VaultError> {
    let mut bytes = [0u8; LEN];
    getrandom::fill(&mut bytes).map_err(|_| VaultError::Random)?;
    Ok(bytes)
}

pub fn random_key() -> Result<Key, VaultError> {
    Ok(Zeroizing::new(random_bytes::<KEY_LEN>()?))
}

pub fn encrypt(key: &Key, nonce: &[u8; NONCE_LEN], plaintext: &[u8], associated_data: &[u8]) -> Result<Vec<u8>, VaultError> {
    build_cipher(key)?
        .encrypt(nonce.into(), Payload { msg: plaintext, aad: associated_data })
        .map_err(|_| VaultError::Encrypt)
}

pub fn decrypt(key: &Key, nonce: &[u8; NONCE_LEN], ciphertext: &[u8], associated_data: &[u8]) -> Result<Zeroizing<Vec<u8>>, VaultError> {
    build_cipher(key)?
        .decrypt(nonce.into(), Payload { msg: ciphertext, aad: associated_data })
        .map(Zeroizing::new)
        .map_err(|_| VaultError::Decrypt)
}
