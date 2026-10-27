//! The 2FAS backup file, exported without a password:
//! `{ "services": [ { "name", "secret", "otp": { "account", "issuer", "digits",
//! "period", "algorithm", "tokenType" } } ] }`. A password-protected backup
//! has `servicesEncrypted` instead of `services`.

use super::account_parts::{build_account, AccountParts};
use super::{ImportBatch, ImportError};
use crate::record::NewAccount;
use crate::vendor::Vendor;
use serde_json::Value;

const TOTP_TOKEN_TYPE: &str = "TOTP";

pub fn import(root: &Value) -> Result<ImportBatch, ImportError> {
    let Some(services) = root.get("services").and_then(Value::as_array) else {
        let is_encrypted = root.get("servicesEncrypted").is_some();
        return Err(if is_encrypted { ImportError::EncryptedExport("2FAS") } else { ImportError::MalformedFile });
    };
    let mut batch = ImportBatch::default();
    for service in services {
        batch.add(import_service(service));
    }
    Ok(batch)
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).filter(|text| !text.trim().is_empty())
}

fn import_service(service: &Value) -> Result<NewAccount, ImportError> {
    let otp = service.get("otp").unwrap_or(&Value::Null);
    let service_name = text(service, "name").unwrap_or("");
    // Older backups omit the token type; those are all time-based.
    if text(otp, "tokenType").is_some_and(|kind| !kind.eq_ignore_ascii_case(TOTP_TOKEN_TYPE)) {
        return Err(ImportError::HotpUnsupported(service_name.to_string()));
    }
    let account = build_account(&AccountParts {
        issuer: text(otp, "issuer").unwrap_or(service_name),
        name: text(otp, "account").unwrap_or(service_name),
        secret_base32: text(service, "secret").unwrap_or(""),
        algorithm: text(otp, "algorithm"),
        digits: otp.get("digits").and_then(Value::as_u64).and_then(|digits| u32::try_from(digits).ok()),
        period_seconds: otp.get("period").and_then(Value::as_u64),
    })?;
    Ok(NewAccount { account, vendor: Vendor::TwoFas })
}
