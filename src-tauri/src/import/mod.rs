//! Importing accounts from the formats other authenticator apps export, or from
//! `otpauth://` and Google Authenticator `otpauth-migration://` links.
//!
//! Anything that cannot be imported (counter-based HOTP, Steam codes, an
//! unreadable entry) is counted and skipped, so one odd entry never costs
//! the user the rest of a file.

mod account_parts;
mod aegis;
mod bitwarden;
mod migration;
mod otpauth_lines;
mod otpauth_uri;
mod protobuf;
mod query;
mod twofas;

use crate::record::NewAccount;
use crate::vendor::Vendor;

const OTPAUTH_PREFIX: &str = "otpauth://";
const MIGRATION_PREFIX: &str = "otpauth-migration://offline?";
/// Above the window's own 5 MB limit, so only input that bypassed the window is refused.
const MAX_IMPORT_TEXT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ImportError {
    #[error("unsupported or malformed URI")]
    BadUri,
    #[error("invalid base64/base32 data")]
    BadEncoding,
    #[error("this is far larger than any authenticator export")]
    TooLarge,
    #[error("malformed migration payload")]
    BadPayload,
    #[error("HOTP (counter-based) accounts are not supported yet: {0}")]
    HotpUnsupported(String),
    #[error("this does not look like a supported export; paste an otpauth link, or use a file from Google Authenticator, Aegis, 2FAS or Bitwarden")]
    UnsupportedFormat,
    #[error("the file is damaged or not in the format it claims to be")]
    MalformedFile,
    #[error("this {0} export is password-protected; export it again without a password and import that file")]
    EncryptedExport(&'static str),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportBatch {
    pub accounts: Vec<NewAccount>,
    /// Entries left out because they are unsupported or unreadable.
    pub skipped_unsupported: usize,
}

impl ImportBatch {
    pub fn add(&mut self, result: Result<NewAccount, ImportError>) {
        match result {
            Ok(account) => self.accounts.push(account),
            Err(_) => self.skipped_unsupported += 1,
        }
    }

    pub fn absorb(&mut self, other: ImportBatch) {
        self.accounts.extend(other.accounts);
        self.skipped_unsupported += other.skipped_unsupported;
    }
}

/// Imports from text: a link, a list of links, or the contents of an export
/// file. `fallback_vendor` tags accounts whose format does not say where they
/// came from (a plain `otpauth://` link).
pub fn import_text(text: &str, fallback_vendor: Vendor) -> Result<ImportBatch, ImportError> {
    if text.len() > MAX_IMPORT_TEXT_BYTES {
        return Err(ImportError::TooLarge);
    }
    let text = text.trim_start_matches('\u{feff}').trim();
    if text.starts_with('{') {
        return import_json(text, fallback_vendor);
    }
    if text.starts_with("otpauth") {
        return otpauth_lines::import(text, fallback_vendor);
    }
    if bitwarden::looks_like_csv(text) {
        return bitwarden::import_csv(text);
    }
    Err(ImportError::UnsupportedFormat)
}

fn import_json(text: &str, fallback_vendor: Vendor) -> Result<ImportBatch, ImportError> {
    let root: serde_json::Value = serde_json::from_str(text).map_err(|_| ImportError::MalformedFile)?;
    if root.get("encrypted").and_then(serde_json::Value::as_bool) == Some(true) {
        return Err(ImportError::EncryptedExport("Bitwarden"));
    }
    if root.get("db").is_some() {
        return aegis::import(&root);
    }
    if root.get("services").is_some() || root.get("servicesEncrypted").is_some() {
        return twofas::import(&root);
    }
    if root.get("items").is_some() {
        return bitwarden::import_json(&root, fallback_vendor);
    }
    Err(ImportError::UnsupportedFormat)
}

#[cfg(test)]
mod tests;
