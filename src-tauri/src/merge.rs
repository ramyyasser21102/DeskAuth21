//! Merging two copies of the vault, for syncing between devices. Pure.
//!
//! Per account id the newest event wins: an edit or a delete, whichever has the
//! later timestamp. A delete beats an edit made at the very same millisecond.
//! The result is the same whichever copy is "local", so devices converge.

use crate::record::{AccountRecord, Tombstone, VaultData};
use std::collections::HashMap;

fn latest_tombstones(local: &VaultData, remote: &VaultData) -> HashMap<String, u64> {
    let mut latest: HashMap<String, u64> = HashMap::new();
    for tombstone in local.tombstones.iter().chain(&remote.tombstones) {
        let entry = latest.entry(tombstone.id.clone()).or_insert(0);
        *entry = (*entry).max(tombstone.deleted_at_ms);
    }
    latest
}

/// Local order first, then accounts only the other copy has, so nothing the
/// user is looking at moves. For an account both copies have, the newer edit wins.
fn newest_records(local: &VaultData, remote: &VaultData) -> Vec<AccountRecord> {
    let mut merged: Vec<AccountRecord> = Vec::new();
    for candidate in local.accounts.iter().chain(&remote.accounts) {
        match merged.iter_mut().find(|record| record.id == candidate.id) {
            Some(existing) if candidate.updated_at_ms > existing.updated_at_ms => *existing = candidate.clone(),
            Some(_) => {}
            None => merged.push(candidate.clone()),
        }
    }
    merged
}

fn survives(record: &AccountRecord, deleted_at: &HashMap<String, u64>) -> bool {
    deleted_at.get(&record.id).is_none_or(|deleted_at_ms| record.updated_at_ms > *deleted_at_ms)
}

pub fn merge(local: &VaultData, remote: &VaultData) -> VaultData {
    let deleted_at = latest_tombstones(local, remote);
    let accounts: Vec<AccountRecord> =
        newest_records(local, remote).into_iter().filter(|record| survives(record, &deleted_at)).collect();
    let mut tombstones: Vec<Tombstone> = deleted_at
        .into_iter()
        .filter(|(id, _)| !accounts.iter().any(|record| &record.id == id))
        .map(|(id, deleted_at_ms)| Tombstone { id, deleted_at_ms })
        .collect();
    tombstones.sort_by(|a, b| a.id.cmp(&b.id));
    VaultData { accounts, tombstones }
}

/// Equal as sets, ignoring order, so a reordering alone is not treated as a change.
pub fn has_same_content(first: &VaultData, second: &VaultData) -> bool {
    let by_id = |data: &VaultData| -> HashMap<String, AccountRecord> {
        data.accounts.iter().map(|record| (record.id.clone(), record.clone())).collect()
    };
    let tombstones = |data: &VaultData| -> HashMap<String, u64> {
        data.tombstones.iter().map(|tombstone| (tombstone.id.clone(), tombstone.deleted_at_ms)).collect()
    };
    by_id(first) == by_id(second) && tombstones(first) == tombstones(second)
}

#[cfg(test)]
mod tests;
