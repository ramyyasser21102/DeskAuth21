use super::*;
use crate::otp::{Account, Algorithm};

fn record(secret: &str, name: &str, updated_at_ms: u64) -> AccountRecord {
    let account = Account {
        issuer: String::new(),
        name: name.into(),
        secret: secret.as_bytes().to_vec(),
        algorithm: Algorithm::Sha1,
        digits: 6,
        period_seconds: 30,
    };
    AccountRecord::new(account, updated_at_ms)
}

fn gone(record: &AccountRecord, deleted_at_ms: u64) -> Tombstone {
    Tombstone { id: record.id.clone(), deleted_at_ms }
}

fn data(accounts: Vec<AccountRecord>, tombstones: Vec<Tombstone>) -> VaultData {
    VaultData { accounts, tombstones }
}

fn names(data: &VaultData) -> Vec<&str> {
    data.accounts.iter().map(|record| record.account.name.as_str()).collect()
}

#[test]
fn accounts_added_on_either_side_are_all_kept() {
    let local = data(vec![record("a", "A", 1)], vec![]);
    let remote = data(vec![record("b", "B", 2)], vec![]);
    assert_eq!(names(&merge(&local, &remote)), ["A", "B"]);
}

#[test]
fn local_order_is_kept_and_newcomers_are_appended() {
    let local = data(vec![record("c", "C", 1), record("a", "A", 1)], vec![]);
    let remote = data(vec![record("a", "A", 1), record("b", "B", 1)], vec![]);
    assert_eq!(names(&merge(&local, &remote)), ["C", "A", "B"]);
}

#[test]
fn the_newer_edit_of_the_same_account_wins() {
    let older = record("a", "Old name", 10);
    let newer = record("a", "New name", 20);
    let merged = merge(&data(vec![older.clone()], vec![]), &data(vec![newer.clone()], vec![]));
    assert_eq!(names(&merged), ["New name"]);
    let flipped = merge(&data(vec![newer], vec![]), &data(vec![older], vec![]));
    assert_eq!(names(&flipped), ["New name"]);
}

#[test]
fn a_delete_removes_the_account_on_the_other_device_too() {
    let account = record("a", "A", 10);
    let deleter = data(vec![], vec![gone(&account, 20)]);
    let holder = data(vec![account], vec![]);
    let merged = merge(&holder, &deleter);
    assert!(merged.accounts.is_empty());
    assert_eq!(merged.tombstones.len(), 1);
}

#[test]
fn an_edit_made_after_a_delete_brings_the_account_back() {
    let account = record("a", "A", 10);
    let deleted = data(vec![], vec![gone(&account, 20)]);
    let reimported = data(vec![record("a", "A again", 30)], vec![]);
    let merged = merge(&deleted, &reimported);
    assert_eq!(names(&merged), ["A again"]);
    assert!(merged.tombstones.is_empty());
}

#[test]
fn an_edit_made_before_a_delete_does_not_survive_it() {
    let account = record("a", "A", 10);
    let merged = merge(&data(vec![record("a", "Edited", 15)], vec![]), &data(vec![], vec![gone(&account, 20)]));
    assert!(merged.accounts.is_empty());
}

#[test]
fn a_delete_wins_a_tie_with_an_edit() {
    let account = record("a", "A", 10);
    let merged = merge(&data(vec![record("a", "Edited", 20)], vec![]), &data(vec![], vec![gone(&account, 20)]));
    assert!(merged.accounts.is_empty());
}

#[test]
fn merging_is_commutative_so_devices_converge() {
    let a = record("a", "A", 5);
    let first = data(vec![a.clone(), record("b", "B", 6)], vec![gone(&record("x", "X", 1), 9)]);
    let second = data(vec![record("a", "A2", 8), record("c", "C", 7)], vec![gone(&a, 7)]);
    assert!(has_same_content(&merge(&first, &second), &merge(&second, &first)));
}

#[test]
fn merging_is_idempotent() {
    let first = data(vec![record("a", "A", 5)], vec![gone(&record("x", "X", 1), 9)]);
    let second = data(vec![record("b", "B", 6)], vec![]);
    let once = merge(&first, &second);
    assert!(has_same_content(&merge(&once, &second), &once));
    assert!(has_same_content(&merge(&once, &once), &once));
}

#[test]
fn only_the_newest_tombstone_per_account_is_kept() {
    let account = record("a", "A", 1);
    let merged = merge(&data(vec![], vec![gone(&account, 5)]), &data(vec![], vec![gone(&account, 9)]));
    assert_eq!(merged.tombstones, vec![gone(&account, 9)]);
}

#[test]
fn same_content_ignores_order_but_notices_real_differences() {
    let a = record("a", "A", 1);
    let b = record("b", "B", 1);
    assert!(has_same_content(&data(vec![a.clone(), b.clone()], vec![]), &data(vec![b.clone(), a.clone()], vec![])));
    assert!(!has_same_content(&data(vec![a.clone()], vec![]), &data(vec![a.clone(), b], vec![])));
    assert!(!has_same_content(&data(vec![a.clone()], vec![]), &data(vec![record("a", "A", 2)], vec![])));
}
