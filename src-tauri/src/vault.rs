//! Encrypted on-disk vault.
//!
//! File layout (all integers big-endian):
//!
//! ```text
//! "DA21" | version u8 | m_kib u32 | t u32 | p u32 | salt[16] | nonce[24] | ciphertext+tag
//! ```
//!
//! The key is derived from the passphrase with Argon2id using the parameters
//! stored in the header. The whole header is bound to the ciphertext as AEAD
//! associated data, so tampering with the KDF parameters or salt fails
//! authentication. The plaintext is the JSON-serialised account list.

use crate::otp::Account;
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::XChaCha20Poly1305;
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"DA21";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const HEADER_LEN: usize = 4 + 1 + 12 + SALT_LEN + NONCE_LEN;

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("not a DeskAuth21 vault, or an unsupported version")]
    BadFormat,
    #[error("wrong passphrase or corrupted vault")]
    Decrypt,
    #[error("invalid key derivation parameters")]
    BadParams,
    #[error("could not gather randomness from the OS")]
    Random,
    #[error("vault contents are not valid: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl KdfParams {
    /// 64 MiB, 3 passes, 1 lane: a sensible interactive default for desktops.
    pub const DEFAULT: Self = Self { m_kib: 64 * 1024, t: 3, p: 1 };
}

/// An unlocked vault: the derived key plus what is needed to re-seal quickly
/// (the slow KDF runs once per unlock, not once per save).
pub struct Session {
    key: Zeroizing<[u8; 32]>,
    salt: [u8; SALT_LEN],
    params: KdfParams,
}

fn derive(passphrase: &str, salt: &[u8], params: KdfParams) -> Result<Zeroizing<[u8; 32]>, VaultError> {
    let p = Params::new(params.m_kib, params.t, params.p, Some(32)).map_err(|_| VaultError::BadParams)?;
    let mut key = Zeroizing::new([0u8; 32]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, p)
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|_| VaultError::BadParams)?;
    Ok(key)
}

fn random<const N: usize>() -> Result<[u8; N], VaultError> {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).map_err(|_| VaultError::Random)?;
    Ok(buf)
}

impl Session {
    /// Start a brand-new vault with a fresh salt.
    pub fn create(passphrase: &str, params: KdfParams) -> Result<Self, VaultError> {
        let salt = random::<SALT_LEN>()?;
        Ok(Self { key: derive(passphrase, &salt, params)?, salt, params })
    }

    /// Encrypt the account list. Uses a fresh nonce every call.
    pub fn seal(&self, accounts: &[Account]) -> Result<Vec<u8>, VaultError> {
        let nonce = random::<NONCE_LEN>()?;
        let mut out = Vec::with_capacity(HEADER_LEN + 64);
        out.extend_from_slice(MAGIC);
        out.push(VERSION);
        out.extend_from_slice(&self.params.m_kib.to_be_bytes());
        out.extend_from_slice(&self.params.t.to_be_bytes());
        out.extend_from_slice(&self.params.p.to_be_bytes());
        out.extend_from_slice(&self.salt);
        out.extend_from_slice(&nonce);

        let plaintext = Zeroizing::new(serde_json::to_vec(accounts)?);
        let cipher = XChaCha20Poly1305::new_from_slice(self.key.as_ref()).map_err(|_| VaultError::BadParams)?;
        let ct = cipher
            .encrypt((&nonce).into(), Payload { msg: &plaintext, aad: &out[..HEADER_LEN - NONCE_LEN] })
            .map_err(|_| VaultError::Decrypt)?;
        out.extend_from_slice(&ct);
        Ok(out)
    }

    /// Decrypt a vault file, returning the live session and its accounts.
    pub fn open(bytes: &[u8], passphrase: &str) -> Result<(Self, Vec<Account>), VaultError> {
        if bytes.len() < HEADER_LEN || &bytes[..4] != MAGIC || bytes[4] != VERSION {
            return Err(VaultError::BadFormat);
        }
        let u32_at = |i: usize| u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap());
        let params = KdfParams { m_kib: u32_at(5), t: u32_at(9), p: u32_at(13) };
        // Refuse absurd costs from a hostile file before spending the memory.
        if params.m_kib > 1024 * 1024 || params.t > 64 || params.p > 16 {
            return Err(VaultError::BadParams);
        }
        let salt: [u8; SALT_LEN] = bytes[17..17 + SALT_LEN].try_into().unwrap();
        let nonce: [u8; NONCE_LEN] = bytes[17 + SALT_LEN..HEADER_LEN].try_into().unwrap();

        let key = derive(passphrase, &salt, params)?;
        let cipher = XChaCha20Poly1305::new_from_slice(key.as_ref()).map_err(|_| VaultError::BadParams)?;
        let plaintext = Zeroizing::new(
            cipher
                .decrypt(
                    (&nonce).into(),
                    Payload { msg: &bytes[HEADER_LEN..], aad: &bytes[..HEADER_LEN - NONCE_LEN] },
                )
                .map_err(|_| VaultError::Decrypt)?,
        );
        let accounts = serde_json::from_slice(&plaintext)?;
        Ok((Self { key, salt, params }, accounts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::Algorithm as OtpAlgorithm;

    // Cheap parameters so the tests stay fast; production uses KdfParams::DEFAULT.
    const FAST: KdfParams = KdfParams { m_kib: 8, t: 1, p: 1 };

    fn sample() -> Vec<Account> {
        vec![Account {
            issuer: "Acme".into(),
            name: "a@b".into(),
            secret: b"12345678901234567890".to_vec(),
            algorithm: OtpAlgorithm::Sha1,
            digits: 6,
            period_seconds: 30,
        }]
    }

    #[test]
    fn roundtrip() {
        let s = Session::create("hunter2", FAST).unwrap();
        let bytes = s.seal(&sample()).unwrap();
        let (_, accounts) = Session::open(&bytes, "hunter2").unwrap();
        assert_eq!(accounts, sample());
    }

    #[test]
    fn secret_is_not_in_plaintext() {
        let s = Session::create("pw", FAST).unwrap();
        let bytes = s.seal(&sample()).unwrap();
        assert!(!bytes.windows(4).any(|w| w == b"Acme"));
    }

    #[test]
    fn wrong_passphrase_fails() {
        let bytes = Session::create("right", FAST).unwrap().seal(&sample()).unwrap();
        assert!(matches!(Session::open(&bytes, "wrong"), Err(VaultError::Decrypt)));
    }

    #[test]
    fn tampering_is_detected() {
        let mut bytes = Session::create("pw", FAST).unwrap().seal(&sample()).unwrap();
        // Flip a ciphertext bit, then separately a KDF-parameter bit (bound as AAD).
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert!(Session::open(&bytes, "pw").is_err());
        bytes[last] ^= 1;
        bytes[8] ^= 1;
        assert!(Session::open(&bytes, "pw").is_err());
    }

    #[test]
    fn reseal_uses_fresh_nonce_and_still_opens() {
        let s = Session::create("pw", FAST).unwrap();
        let (a, b) = (s.seal(&sample()).unwrap(), s.seal(&sample()).unwrap());
        assert_ne!(a, b);
        assert!(Session::open(&b, "pw").is_ok());
    }

    #[test]
    fn rejects_garbage_and_hostile_params() {
        assert!(matches!(Session::open(b"nope", "pw"), Err(VaultError::BadFormat)));
        let mut bytes = Session::create("pw", FAST).unwrap().seal(&sample()).unwrap();
        bytes[5..9].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(matches!(Session::open(&bytes, "pw"), Err(VaultError::BadParams)));
    }
}
