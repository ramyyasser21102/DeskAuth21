//! HOTP/TOTP (RFC 4226 / RFC 6238).

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

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
    pub secret: Vec<u8>,
    pub algorithm: Algorithm,
    pub digits: u32,
    pub period: u64,
}

fn mac<M: Mac + hmac::digest::KeyInit>(secret: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut m = <M as hmac::digest::KeyInit>::new_from_slice(secret).expect("HMAC accepts any key length");
    m.update(msg);
    m.finalize().into_bytes().to_vec()
}

pub fn hotp(secret: &[u8], counter: u64, algorithm: Algorithm, digits: u32) -> String {
    let msg = counter.to_be_bytes();
    let h = match algorithm {
        Algorithm::Sha1 => mac::<Hmac<Sha1>>(secret, &msg),
        Algorithm::Sha256 => mac::<Hmac<Sha256>>(secret, &msg),
        Algorithm::Sha512 => mac::<Hmac<Sha512>>(secret, &msg),
    };
    let offset = (h[h.len() - 1] & 0x0f) as usize;
    let bin = u32::from_be_bytes([h[offset] & 0x7f, h[offset + 1], h[offset + 2], h[offset + 3]]);
    format!("{:0width$}", bin % 10u32.pow(digits), width = digits as usize)
}

impl Account {
    pub fn code_at(&self, unix_secs: u64) -> String {
        hotp(&self.secret, unix_secs / self.period, self.algorithm, self.digits)
    }

    pub fn seconds_left(&self, unix_secs: u64) -> u64 {
        self.period - unix_secs % self.period
    }

    pub fn label(&self) -> String {
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

    // RFC 6238 appendix B test vectors (8 digits).
    #[test]
    fn rfc6238_vectors() {
        let k1 = b"12345678901234567890";
        let k256 = b"12345678901234567890123456789012";
        let k512 = b"1234567890123456789012345678901234567890123456789012345678901234";
        assert_eq!(hotp(k1, 59 / 30, Algorithm::Sha1, 8), "94287082");
        assert_eq!(hotp(k256, 59 / 30, Algorithm::Sha256, 8), "46119246");
        assert_eq!(hotp(k512, 59 / 30, Algorithm::Sha512, 8), "90693936");
        assert_eq!(hotp(k1, 1111111109 / 30, Algorithm::Sha1, 8), "07081804");
    }
}
