//! Bitwarden's unencrypted export, as JSON (`items[].login.totp`) or CSV
//! (a `login_totp` column). The value is an `otpauth://` link or a bare secret;
//! Steam codes are not supported.

use super::account_parts::{build_account, AccountParts};
use super::otpauth_uri::parse_otpauth_uri;
use super::{ImportBatch, ImportError};
use crate::otp::Account;
use crate::record::NewAccount;
use crate::vendor::Vendor;
use serde_json::Value;

const OTPAUTH_SCHEME: &str = "otpauth://";
const STEAM_SCHEME: &str = "steam://";
const CSV_TOTP_COLUMN: &str = "login_totp";

pub fn looks_like_csv(text: &str) -> bool {
    text.lines().next().is_some_and(|header| header.contains(CSV_TOTP_COLUMN) && header.contains("name"))
}

/// One export entry: the item's title, the username, and its TOTP value.
fn account_from_login(title: &str, username: &str, totp: &str) -> Result<Account, ImportError> {
    let totp = totp.trim();
    if totp.starts_with(STEAM_SCHEME) {
        return Err(ImportError::HotpUnsupported(title.to_string()));
    }
    if totp.starts_with(OTPAUTH_SCHEME) {
        let mut account = parse_otpauth_uri(totp)?;
        if account.issuer.is_empty() {
            account.issuer = title.trim().to_string();
        }
        return Ok(account);
    }
    let name = if username.trim().is_empty() { title } else { username };
    build_account(&AccountParts { issuer: title, name, secret_base32: totp, algorithm: None, digits: None, period_seconds: None })
}

fn as_new_account(account: Result<Account, ImportError>) -> Result<NewAccount, ImportError> {
    account.map(|account| NewAccount { account, vendor: Vendor::Bitwarden })
}

pub fn import_json(root: &Value, _fallback_vendor: Vendor) -> Result<ImportBatch, ImportError> {
    let items = root.get("items").and_then(Value::as_array).ok_or(ImportError::MalformedFile)?;
    let mut batch = ImportBatch::default();
    for item in items {
        let totp = item.pointer("/login/totp").and_then(Value::as_str).unwrap_or("");
        if totp.trim().is_empty() {
            continue; // an ordinary login with no one-time code
        }
        let title = item.get("name").and_then(Value::as_str).unwrap_or("");
        let username = item.pointer("/login/username").and_then(Value::as_str).unwrap_or("");
        batch.add(as_new_account(account_from_login(title, username, totp)));
    }
    Ok(batch)
}

pub fn import_csv(text: &str) -> Result<ImportBatch, ImportError> {
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(text.as_bytes());
    let headers = reader.headers().map_err(|_| ImportError::MalformedFile)?.clone();
    let column = |name: &str| headers.iter().position(|header| header == name);
    let (Some(name_column), Some(totp_column)) = (column("name"), column(CSV_TOTP_COLUMN)) else {
        return Err(ImportError::MalformedFile);
    };
    let username_column = column("login_username");
    let mut batch = ImportBatch::default();
    for record in reader.records() {
        let record = record.map_err(|_| ImportError::MalformedFile)?;
        let totp = record.get(totp_column).unwrap_or("");
        if totp.trim().is_empty() {
            continue;
        }
        let username = username_column.and_then(|index| record.get(index)).unwrap_or("");
        batch.add(as_new_account(account_from_login(record.get(name_column).unwrap_or(""), username, totp)));
    }
    Ok(batch)
}
