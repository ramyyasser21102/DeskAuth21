//! Parser for Google Authenticator's `otpauth-migration://offline?data=...` export.
//!
//! Wire schema (MigrationPayload): field 1 repeats `OtpParameters` messages.

use super::protobuf::{Field, Reader};
use super::query::query_param;
use super::{ImportBatch, ImportError};
use crate::otp::{Account, Algorithm};
use crate::record::NewAccount;
use crate::secret::Secret;
use crate::vendor::Vendor;
use base64::Engine;

const PAYLOAD_FIELD_OTP_PARAMETERS: u64 = 1;

const PARAM_FIELD_SECRET: u64 = 1;
const PARAM_FIELD_NAME: u64 = 2;
const PARAM_FIELD_ISSUER: u64 = 3;
const PARAM_FIELD_ALGORITHM: u64 = 4;
const PARAM_FIELD_DIGITS: u64 = 5;
const PARAM_FIELD_OTP_TYPE: u64 = 6;

const ALGORITHM_SHA256: u64 = 2;
const ALGORITHM_SHA512: u64 = 3;
const DIGITS_EIGHT: u64 = 2;
const OTP_TYPE_HOTP: u64 = 1;

const SIX_DIGITS: u32 = 6;
const EIGHT_DIGITS: u32 = 8;
const MIGRATION_PERIOD_SECONDS: u64 = 30;

#[derive(Default)]
struct RawOtpParameters {
    secret: Secret,
    name: String,
    issuer: String,
    algorithm_code: u64,
    digits_code: u64,
    otp_type_code: u64,
}

/// Entries that cannot be imported (counter-based HOTP, or unreadable) are counted
/// rather than failing the whole export.
pub fn parse_migration_query(query: &str) -> Result<ImportBatch, ImportError> {
    let encoded = query_param(query, "data").ok_or(ImportError::BadUri)?;
    // A raw '+' in a query string may arrive as a space.
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.replace(' ', "+"))
        .map_err(|_| ImportError::BadEncoding)?;
    parse_migration_payload(&bytes)
}

fn parse_migration_payload(bytes: &[u8]) -> Result<ImportBatch, ImportError> {
    let mut reader = Reader::new(bytes);
    let mut batch = ImportBatch::default();
    while let Some((number, field)) = reader.next_field()? {
        if let (PAYLOAD_FIELD_OTP_PARAMETERS, Field::Bytes(entry)) = (number, field) {
            let account = read_otp_parameters(entry).and_then(account_from_parameters);
            batch.add(account.map(|account| NewAccount { account, vendor: Vendor::GoogleAuthenticator }));
        }
    }
    Ok(batch)
}

fn read_otp_parameters(bytes: &[u8]) -> Result<RawOtpParameters, ImportError> {
    let mut reader = Reader::new(bytes);
    let mut parameters = RawOtpParameters::default();
    while let Some((number, field)) = reader.next_field()? {
        parameters.apply_field(number, field);
    }
    Ok(parameters)
}

impl RawOtpParameters {
    fn apply_field(&mut self, number: u64, field: Field) {
        match (number, field) {
            (PARAM_FIELD_SECRET, Field::Bytes(bytes)) => self.secret = Secret::new(bytes.to_vec()),
            (PARAM_FIELD_NAME, Field::Bytes(bytes)) => self.name = lossy_text(bytes),
            (PARAM_FIELD_ISSUER, Field::Bytes(bytes)) => self.issuer = lossy_text(bytes),
            (PARAM_FIELD_ALGORITHM, Field::Varint(code)) => self.algorithm_code = code,
            (PARAM_FIELD_DIGITS, Field::Varint(code)) => self.digits_code = code,
            (PARAM_FIELD_OTP_TYPE, Field::Varint(code)) => self.otp_type_code = code,
            _ => {}
        }
    }
}

fn lossy_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn account_from_parameters(raw: RawOtpParameters) -> Result<Account, ImportError> {
    if raw.otp_type_code == OTP_TYPE_HOTP {
        return Err(ImportError::HotpUnsupported(raw.name));
    }
    Ok(Account {
        name: strip_issuer_prefix(&raw.name, &raw.issuer),
        algorithm: algorithm_from_code(raw.algorithm_code),
        digits: digits_from_code(raw.digits_code),
        issuer: raw.issuer,
        secret: raw.secret,
        period_seconds: MIGRATION_PERIOD_SECONDS,
    })
}

/// Google stores "Issuer:account" in the name field when an issuer is set.
fn strip_issuer_prefix(name: &str, issuer: &str) -> String {
    let prefix = format!("{issuer}:");
    match name.strip_prefix(&prefix) {
        Some(rest) if !issuer.is_empty() => rest.trim().to_string(),
        _ => name.to_string(),
    }
}

fn algorithm_from_code(code: u64) -> Algorithm {
    match code {
        ALGORITHM_SHA256 => Algorithm::Sha256,
        ALGORITHM_SHA512 => Algorithm::Sha512,
        _ => Algorithm::Sha1,
    }
}

fn digits_from_code(code: u64) -> u32 {
    if code == DIGITS_EIGHT { EIGHT_DIGITS } else { SIX_DIGITS }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &[u8] = b"12345678901234567890";

    fn length_delimited(field_number: u8, bytes: &[u8]) -> Vec<u8> {
        let tag = (field_number << 3) | 2;
        [&[tag, bytes.len() as u8], bytes].concat()
    }

    fn varint(field_number: u8, value: u8) -> Vec<u8> {
        vec![field_number << 3, value]
    }

    fn entry(name: &str, issuer: &str, otp_type: u8) -> Vec<u8> {
        [
            length_delimited(1, SEED),
            length_delimited(2, name.as_bytes()),
            length_delimited(3, issuer.as_bytes()),
            varint(4, 1),
            varint(5, 1),
            varint(6, otp_type),
        ]
        .concat()
    }

    fn query_for(entries: &[Vec<u8>]) -> String {
        let payload: Vec<u8> = entries.iter().flat_map(|e| length_delimited(1, e)).collect();
        let encoded = base64::engine::general_purpose::STANDARD.encode(payload);
        format!("data={}", urlencoding::encode(&encoded))
    }

    #[test]
    fn imports_every_entry_in_the_payload_tagged_as_google_authenticator() {
        let query = query_for(&[entry("a@b", "Acme", 2), entry("c@d", "Other", 2)]);
        let batch = parse_migration_query(&query).unwrap();
        assert_eq!(batch.accounts.len(), 2);
        let first = &batch.accounts[0];
        assert_eq!((first.account.issuer.as_str(), first.account.name.as_str()), ("Acme", "a@b"));
        assert_eq!(first.vendor, Vendor::GoogleAuthenticator);
        assert_eq!(batch.accounts[1].account.secret.as_bytes(), SEED);
    }

    #[test]
    fn strips_the_issuer_prefix_only_when_it_matches() {
        assert_eq!(strip_issuer_prefix("Acme:a@b", "Acme"), "a@b");
        assert_eq!(strip_issuer_prefix("Other:a@b", "Acme"), "Other:a@b");
        assert_eq!(strip_issuer_prefix("Acme:a@b", ""), "Acme:a@b");
    }

    #[test]
    fn counts_hotp_entries_as_unsupported_without_losing_the_rest() {
        let query = query_for(&[entry("counter", "Acme", 1), entry("c@d", "Other", 2)]);
        let batch = parse_migration_query(&query).unwrap();
        assert_eq!((batch.accounts.len(), batch.skipped_unsupported), (1, 1));
    }

    #[test]
    fn rejects_missing_or_corrupt_data() {
        assert_eq!(parse_migration_query(""), Err(ImportError::BadUri));
        assert_eq!(parse_migration_query("data=%%%"), Err(ImportError::BadEncoding));
    }
}
