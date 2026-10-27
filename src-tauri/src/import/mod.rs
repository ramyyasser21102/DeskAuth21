//! Import accounts from `otpauth://` and Google Authenticator
//! `otpauth-migration://` URIs.

mod migration;
mod otpauth_uri;
mod protobuf;
mod query;

use crate::otp::Account;

const OTPAUTH_PREFIX: &str = "otpauth://";
const MIGRATION_PREFIX: &str = "otpauth-migration://offline?";

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ImportError {
    #[error("unsupported or malformed URI")]
    BadUri,
    #[error("invalid base64/base32 data")]
    BadEncoding,
    #[error("malformed migration payload")]
    BadPayload,
    #[error("HOTP (counter-based) accounts are not supported yet: {0}")]
    HotpUnsupported(String),
}

pub fn import_uri(uri: &str) -> Result<Vec<Account>, ImportError> {
    let uri = uri.trim();
    if let Some(query) = uri.strip_prefix(MIGRATION_PREFIX) {
        return migration::parse_migration_query(query);
    }
    if uri.starts_with(OTPAUTH_PREFIX) {
        return otpauth_uri::parse_otpauth_uri(uri).map(|account| vec![account]);
    }
    Err(ImportError::BadUri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_schemes() {
        assert_eq!(import_uri("https://example.com"), Err(ImportError::BadUri));
        assert_eq!(import_uri(""), Err(ImportError::BadUri));
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let uri = "  otpauth://totp/a?secret=GEZDGNBVGY3TQOJQ \n";
        assert_eq!(import_uri(uri).unwrap().len(), 1);
    }
}
