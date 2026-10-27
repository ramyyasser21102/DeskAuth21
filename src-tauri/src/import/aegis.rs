//! Aegis Authenticator's plain JSON export:
//! `{ "db": { "entries": [ { "type", "name", "issuer", "info": { "secret", "algo", "digits", "period" } } ] } }`.
//! An encrypted export has a string instead of an object in `db`.

use super::account_parts::{build_account, AccountParts};
use super::{ImportBatch, ImportError};
use crate::record::NewAccount;
use crate::vendor::Vendor;
use serde_json::Value;

const TOTP_TYPE: &str = "totp";

pub fn import(root: &Value) -> Result<ImportBatch, ImportError> {
    let db = root.get("db").ok_or(ImportError::MalformedFile)?;
    if db.is_string() {
        return Err(ImportError::EncryptedExport("Aegis"));
    }
    let entries = db.get("entries").and_then(Value::as_array).ok_or(ImportError::MalformedFile)?;
    let mut batch = ImportBatch::default();
    for entry in entries {
        batch.add(import_entry(entry));
    }
    Ok(batch)
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn import_entry(entry: &Value) -> Result<NewAccount, ImportError> {
    if text(entry, "type") != TOTP_TYPE {
        return Err(ImportError::HotpUnsupported(text(entry, "name").to_string()));
    }
    let info = entry.get("info").ok_or(ImportError::MalformedFile)?;
    let account = build_account(&AccountParts {
        issuer: text(entry, "issuer"),
        name: text(entry, "name"),
        secret_base32: text(info, "secret"),
        algorithm: info.get("algo").and_then(Value::as_str),
        digits: info.get("digits").and_then(Value::as_u64).and_then(|digits| u32::try_from(digits).ok()),
        period_seconds: info.get("period").and_then(Value::as_u64),
    })?;
    Ok(NewAccount { account, vendor: Vendor::Aegis })
}
