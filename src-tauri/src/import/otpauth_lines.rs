//! A link, or a text file with one link per line (what Ente Auth and several
//! other apps export). Google Authenticator export links are recognised too.

use super::{migration, otpauth_uri, ImportBatch, ImportError, MIGRATION_PREFIX, OTPAUTH_PREFIX};
use crate::record::NewAccount;
use crate::vendor::Vendor;

pub fn import(text: &str, fallback_vendor: Vendor) -> Result<ImportBatch, ImportError> {
    let mut batch = ImportBatch::default();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        batch.absorb(import_line(line, fallback_vendor)?);
    }
    Ok(batch)
}

fn import_line(line: &str, fallback_vendor: Vendor) -> Result<ImportBatch, ImportError> {
    if let Some(query) = line.strip_prefix(MIGRATION_PREFIX) {
        return migration::parse_migration_query(query);
    }
    let mut batch = ImportBatch::default();
    if line.starts_with(OTPAUTH_PREFIX) {
        let account = otpauth_uri::parse_otpauth_uri(line);
        batch.add(account.map(|account| NewAccount { account, vendor: fallback_vendor }));
    } else {
        batch.skipped_unsupported += 1;
    }
    Ok(batch)
}
