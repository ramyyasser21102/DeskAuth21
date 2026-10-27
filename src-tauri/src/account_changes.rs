//! Adding, renaming and deleting accounts. Pure: each returns a new `VaultData`
//! and the time is passed in.

use crate::record::{AccountRecord, NewAccount, Tombstone, VaultData};
use crate::vendor::Vendor;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ChangeError {
    #[error("no account with id {0}")]
    AccountNotFound(String),
    #[error("an account needs a name")]
    EmptyName,
}

/// What the user can edit about an account: who issued it, which login it is
/// for, and which app it came from.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AccountDetails {
    pub issuer: String,
    pub name: String,
    pub vendor: Vendor,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AddReport {
    pub added: usize,
    pub skipped_duplicates: usize,
}

fn newest_timestamp(data: &VaultData) -> u64 {
    let edits = data.accounts.iter().map(|record| record.updated_at_ms);
    let deletes = data.tombstones.iter().map(|tombstone| tombstone.deleted_at_ms);
    edits.chain(deletes).max().unwrap_or(0)
}

/// Strictly later than anything already in the vault, even if this device's
/// clock is behind another device's, so a new change always wins over what we have seen.
fn change_timestamp(data: &VaultData, now_ms: u64) -> u64 {
    now_ms.max(newest_timestamp(data) + 1)
}

fn find_index(data: &VaultData, id: &str) -> Result<usize, ChangeError> {
    data.accounts
        .iter()
        .position(|record| record.id == id)
        .ok_or_else(|| ChangeError::AccountNotFound(id.to_string()))
}

/// Skips accounts already in the vault (same secret) and repeats within the batch.
/// An account that was deleted can be added again; it counts as new.
pub fn add_accounts(data: &VaultData, imported: Vec<NewAccount>, now_ms: u64) -> (VaultData, AddReport) {
    let timestamp = change_timestamp(data, now_ms);
    let mut updated = data.clone();
    let mut skipped_duplicates = 0;
    for new_account in imported {
        let record = AccountRecord::with_vendor(new_account, timestamp);
        if updated.accounts.iter().any(|existing| existing.id == record.id) {
            skipped_duplicates += 1;
            continue;
        }
        updated.tombstones.retain(|tombstone| tombstone.id != record.id);
        updated.accounts.push(record);
    }
    let added = updated.accounts.len() - data.accounts.len();
    (updated, AddReport { added, skipped_duplicates })
}

pub fn rename_account(data: &VaultData, id: &str, details: &AccountDetails, now_ms: u64) -> Result<VaultData, ChangeError> {
    let name = details.name.trim();
    if name.is_empty() {
        return Err(ChangeError::EmptyName);
    }
    let index = find_index(data, id)?;
    let mut updated = data.clone();
    let record = &mut updated.accounts[index];
    record.account.issuer = details.issuer.trim().to_string();
    record.vendor = details.vendor;
    record.account.name = name.to_string();
    record.updated_at_ms = change_timestamp(data, now_ms);
    Ok(updated)
}

pub fn delete_account(data: &VaultData, id: &str, now_ms: u64) -> Result<VaultData, ChangeError> {
    let index = find_index(data, id)?;
    let mut updated = data.clone();
    updated.accounts.remove(index);
    updated.tombstones.push(Tombstone { id: id.to_string(), deleted_at_ms: change_timestamp(data, now_ms) });
    Ok(updated)
}

#[cfg(test)]
mod tests;
