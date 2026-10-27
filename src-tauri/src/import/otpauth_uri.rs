//! Parser for the standard `otpauth://totp/Issuer:name?secret=...` format.

use super::query::{numeric_param, query_param};
use super::{ImportError, OTPAUTH_PREFIX};
use crate::otp::{Account, Algorithm};

const DEFAULT_DIGITS: u32 = 6;
const DEFAULT_PERIOD_SECONDS: u64 = 30;
const SUPPORTED_DIGITS: std::ops::RangeInclusive<u32> = 6..=8;
const TOTP_KIND: &str = "totp";

struct UriParts<'a> {
    kind: &'a str,
    label: String,
    query: &'a str,
}

pub fn parse_otpauth_uri(uri: &str) -> Result<Account, ImportError> {
    let parts = split_uri(uri)?;
    if parts.kind != TOTP_KIND {
        return Err(ImportError::HotpUnsupported(parts.label));
    }
    let (label_issuer, name) = split_label(&parts.label);
    let account = Account {
        issuer: query_param(parts.query, "issuer").unwrap_or(label_issuer),
        name,
        secret: decode_secret(parts.query)?,
        algorithm: parse_algorithm(parts.query)?,
        digits: numeric_param(parts.query, "digits", DEFAULT_DIGITS),
        period_seconds: numeric_param(parts.query, "period", DEFAULT_PERIOD_SECONDS),
    };
    ensure_supported_timing(&account)?;
    Ok(account)
}

fn split_uri(uri: &str) -> Result<UriParts<'_>, ImportError> {
    let rest = uri.strip_prefix(OTPAUTH_PREFIX).ok_or(ImportError::BadUri)?;
    let (kind_and_label, query) = rest.split_once('?').ok_or(ImportError::BadUri)?;
    let (kind, encoded_label) = kind_and_label.split_once('/').ok_or(ImportError::BadUri)?;
    let label = urlencoding::decode(encoded_label).map_err(|_| ImportError::BadUri)?;
    Ok(UriParts { kind, label: label.into_owned(), query })
}

/// "Issuer:name" becomes ("Issuer", "name"); a bare label has no issuer.
fn split_label(label: &str) -> (String, String) {
    match label.split_once(':') {
        Some((issuer, name)) => (issuer.trim().to_string(), name.trim().to_string()),
        None => (String::new(), label.to_string()),
    }
}

fn decode_secret(query: &str) -> Result<Vec<u8>, ImportError> {
    let encoded = query_param(query, "secret").ok_or(ImportError::BadUri)?;
    let normalized = encoded.replace(' ', "").trim_end_matches('=').to_uppercase();
    base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &normalized)
        .ok_or(ImportError::BadEncoding)
}

fn parse_algorithm(query: &str) -> Result<Algorithm, ImportError> {
    let name = query_param(query, "algorithm").map(|value| value.to_uppercase());
    match name.as_deref() {
        None | Some("SHA1") => Ok(Algorithm::Sha1),
        Some("SHA256") => Ok(Algorithm::Sha256),
        Some("SHA512") => Ok(Algorithm::Sha512),
        Some(_) => Err(ImportError::BadUri),
    }
}

/// A zero period would divide by zero when computing codes, and digit counts
/// outside 6..=8 are not interoperable with real services.
fn ensure_supported_timing(account: &Account) -> Result<(), ImportError> {
    let is_valid = account.period_seconds > 0 && SUPPORTED_DIGITS.contains(&account.digits);
    if is_valid { Ok(()) } else { Err(ImportError::BadUri) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED_BASE32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

    fn uri(query: &str) -> String {
        format!("otpauth://totp/GitHub:me%40x.com?secret={SEED_BASE32}&{query}")
    }

    #[test]
    fn parses_issuer_name_and_secret() {
        let account = parse_otpauth_uri(&uri("issuer=GitHub")).unwrap();
        assert_eq!(account.issuer, "GitHub");
        assert_eq!(account.name, "me@x.com");
        assert_eq!(account.secret, b"12345678901234567890");
    }

    #[test]
    fn falls_back_to_label_issuer_and_defaults() {
        let account = parse_otpauth_uri(&uri("")).unwrap();
        assert_eq!(account.issuer, "GitHub");
        assert_eq!((account.digits, account.period_seconds), (6, 30));
        assert_eq!(account.algorithm, Algorithm::Sha1);
    }

    #[test]
    fn rejects_hotp() {
        let hotp = format!("otpauth://hotp/x?secret={SEED_BASE32}&counter=1");
        assert_eq!(parse_otpauth_uri(&hotp), Err(ImportError::HotpUnsupported("x".into())));
    }

    #[test]
    fn rejects_values_that_would_break_code_generation() {
        assert_eq!(parse_otpauth_uri(&uri("period=0")), Err(ImportError::BadUri));
        assert_eq!(parse_otpauth_uri(&uri("digits=12")), Err(ImportError::BadUri));
        assert_eq!(parse_otpauth_uri(&uri("algorithm=MD5")), Err(ImportError::BadUri));
    }

    #[test]
    fn rejects_secret_that_is_not_base32() {
        let bad = "otpauth://totp/x?secret=!!!";
        assert_eq!(parse_otpauth_uri(bad), Err(ImportError::BadEncoding));
    }
}
