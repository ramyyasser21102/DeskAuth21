//! HOTP/TOTP (RFC 4226 / RFC 6238). Pure: time is always passed in.

use hmac::{Hmac, Mac};
use crate::secret::Secret;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

const LOW_NIBBLE_MASK: u8 = 0x0f;
const CLEAR_SIGN_BIT_MASK: u8 = 0x7f;
const DECIMAL_BASE: u32 = 10;
const TRUNCATED_WORD_LEN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub issuer: String,
    pub name: String,
    pub secret: Secret,
    pub algorithm: Algorithm,
    pub digits: u32,
    pub period_seconds: u64,
}

fn compute_hmac<M: Mac + hmac::digest::KeyInit>(secret: &[u8], message: &[u8]) -> Vec<u8> {
    let mut mac = <M as hmac::digest::KeyInit>::new_from_slice(secret)
        .expect("HMAC accepts keys of any length");
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}

fn hmac_digest(algorithm: Algorithm, secret: &[u8], message: &[u8]) -> Vec<u8> {
    match algorithm {
        Algorithm::Sha1 => compute_hmac::<Hmac<Sha1>>(secret, message),
        Algorithm::Sha256 => compute_hmac::<Hmac<Sha256>>(secret, message),
        Algorithm::Sha512 => compute_hmac::<Hmac<Sha512>>(secret, message),
    }
}

/// RFC 4226 section 5.3: pick 4 bytes at an offset taken from the last nibble.
fn dynamic_truncate(digest: &[u8]) -> u32 {
    let offset = usize::from(digest[digest.len() - 1] & LOW_NIBBLE_MASK);
    let word = &digest[offset..offset + TRUNCATED_WORD_LEN];
    u32::from_be_bytes([word[0] & CLEAR_SIGN_BIT_MASK, word[1], word[2], word[3]])
}

pub fn hotp(secret: &[u8], counter: u64, algorithm: Algorithm, digits: u32) -> String {
    let digest = hmac_digest(algorithm, secret, &counter.to_be_bytes());
    let code = dynamic_truncate(&digest) % DECIMAL_BASE.pow(digits);
    format!("{code:0width$}", width = digits as usize)
}

impl Account {
    /// `period_seconds` must be non-zero; importers reject zero.
    pub fn code_at(&self, unix_seconds: u64) -> String {
        let time_step = unix_seconds / self.period_seconds;
        hotp(self.secret.as_bytes(), time_step, self.algorithm, self.digits)
    }

    pub fn seconds_until_next_code(&self, unix_seconds: u64) -> u64 {
        self.period_seconds - unix_seconds % self.period_seconds
    }

    pub fn display_label(&self) -> String {
        if self.issuer.is_empty() {
            self.name.clone()
        } else {
            format!("{} ({})", self.issuer, self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA1_SEED: &[u8] = b"12345678901234567890";
    const SHA256_SEED: &[u8] = b"12345678901234567890123456789012";
    const SHA512_SEED: &[u8] = b"1234567890123456789012345678901234567890123456789012345678901234";
    const RFC_TIME_STEP_SECONDS: u64 = 30;

    fn rfc_code(seed: &[u8], unix_seconds: u64, algorithm: Algorithm) -> String {
        hotp(seed, unix_seconds / RFC_TIME_STEP_SECONDS, algorithm, 8)
    }

    // RFC 6238 appendix B test vectors (8 digits).
    #[test]
    fn matches_rfc6238_vectors() {
        assert_eq!(rfc_code(SHA1_SEED, 59, Algorithm::Sha1), "94287082");
        assert_eq!(rfc_code(SHA256_SEED, 59, Algorithm::Sha256), "46119246");
        assert_eq!(rfc_code(SHA512_SEED, 59, Algorithm::Sha512), "90693936");
        assert_eq!(rfc_code(SHA1_SEED, 1_111_111_109, Algorithm::Sha1), "07081804");
    }

    fn sample_account() -> Account {
        Account {
            issuer: "Acme".into(),
            name: "a@b".into(),
            secret: Secret::new(SHA1_SEED.to_vec()),
            algorithm: Algorithm::Sha1,
            digits: 6,
            period_seconds: RFC_TIME_STEP_SECONDS,
        }
    }

    #[test]
    fn counts_down_to_the_next_window() {
        let account = sample_account();
        assert_eq!(account.seconds_until_next_code(0), 30);
        assert_eq!(account.seconds_until_next_code(29), 1);
        assert_eq!(account.seconds_until_next_code(30), 30);
    }

    #[test]
    fn label_includes_issuer_only_when_present() {
        let mut account = sample_account();
        assert_eq!(account.display_label(), "Acme (a@b)");
        account.issuer.clear();
        assert_eq!(account.display_label(), "a@b");
    }
}
