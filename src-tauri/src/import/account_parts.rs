//! Turning the pieces of an account, however a file stores them, into a
//! validated `Account`. Every import format goes through here, so they all
//! get the same checks.

use super::ImportError;
use crate::otp::{Account, Algorithm};
use crate::secret::Secret;

const DEFAULT_DIGITS: u32 = 6;
const DEFAULT_PERIOD_SECONDS: u64 = 30;
const SUPPORTED_DIGITS: std::ops::RangeInclusive<u32> = 6..=8;

pub struct AccountParts<'a> {
    pub issuer: &'a str,
    pub name: &'a str,
    pub secret_base32: &'a str,
    pub algorithm: Option<&'a str>,
    pub digits: Option<u32>,
    pub period_seconds: Option<u64>,
}

pub fn decode_base32_secret(text: &str) -> Result<Vec<u8>, ImportError> {
    let normalized = text.replace([' ', '-'], "").trim_end_matches('=').to_uppercase();
    base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &normalized).ok_or(ImportError::BadEncoding)
}

pub fn parse_algorithm_name(name: Option<&str>) -> Result<Algorithm, ImportError> {
    match name.map(str::to_uppercase).as_deref() {
        None | Some("SHA1") => Ok(Algorithm::Sha1),
        Some("SHA256") => Ok(Algorithm::Sha256),
        Some("SHA512") => Ok(Algorithm::Sha512),
        Some(_) => Err(ImportError::BadUri),
    }
}

/// A zero period would divide by zero when computing codes, and digit counts
/// outside 6..=8 are not interoperable with real services.
fn ensure_supported_timing(digits: u32, period_seconds: u64) -> Result<(), ImportError> {
    if period_seconds > 0 && SUPPORTED_DIGITS.contains(&digits) { Ok(()) } else { Err(ImportError::BadUri) }
}

pub fn build_account(parts: &AccountParts) -> Result<Account, ImportError> {
    let digits = parts.digits.unwrap_or(DEFAULT_DIGITS);
    let period_seconds = parts.period_seconds.unwrap_or(DEFAULT_PERIOD_SECONDS);
    ensure_supported_timing(digits, period_seconds)?;
    Ok(Account {
        issuer: parts.issuer.trim().to_string(),
        name: parts.name.trim().to_string(),
        secret: Secret::new(decode_base32_secret(parts.secret_base32)?),
        algorithm: parse_algorithm_name(parts.algorithm)?,
        digits,
        period_seconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(secret: &'static str) -> AccountParts<'static> {
        AccountParts { issuer: " Acme ", name: "a@b", secret_base32: secret, algorithm: None, digits: None, period_seconds: None }
    }

    #[test]
    fn builds_an_account_with_defaults_and_trimmed_labels() {
        let account = build_account(&parts("GEZDGNBVGY3TQOJQ")).unwrap();
        assert_eq!((account.issuer.as_str(), account.digits, account.period_seconds), ("Acme", 6, 30));
        assert_eq!(account.secret.as_bytes(), b"1234567890");
    }

    #[test]
    fn accepts_secrets_written_with_spaces_dashes_lowercase_and_padding() {
        let expected = decode_base32_secret("GEZDGNBVGY3TQOJQ").unwrap();
        assert_eq!(decode_base32_secret("gezd gnbv-gy3t qojq====").unwrap(), expected);
    }

    #[test]
    fn rejects_bad_secrets_algorithms_and_timing() {
        assert_eq!(build_account(&parts("!!!")), Err(ImportError::BadEncoding));
        let md5 = AccountParts { algorithm: Some("MD5"), ..parts("GEZDGNBVGY3TQOJQ") };
        assert_eq!(build_account(&md5), Err(ImportError::BadUri));
        let zero_period = AccountParts { period_seconds: Some(0), ..parts("GEZDGNBVGY3TQOJQ") };
        assert_eq!(build_account(&zero_period), Err(ImportError::BadUri));
        let twelve_digits = AccountParts { digits: Some(12), ..parts("GEZDGNBVGY3TQOJQ") };
        assert_eq!(build_account(&twelve_digits), Err(ImportError::BadUri));
    }
}
