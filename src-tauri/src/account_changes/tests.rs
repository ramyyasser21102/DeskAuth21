use super::*;
use crate::merge::merge;
use crate::otp::Algorithm;

fn labels(issuer: &str, name: &str) -> AccountLabels {
    AccountLabels { issuer: issuer.into(), name: name.into() }
}

fn account(secret: &str, name: &str) -> Account {
    Account {
        issuer: "Acme".into(),
        name: name.into(),
        secret: secret.as_bytes().to_vec(),
        algorithm: Algorithm::Sha1,
        digits: 6,
        period_seconds: 30,
    }
}

fn with_accounts(accounts: &[(&str, &str)], now_ms: u64) -> VaultData {
    let imported = accounts.iter().map(|(secret, name)| account(secret, name)).collect();
    add_accounts(&VaultData::default(), imported, now_ms).0
}

fn id_of(secret: &str) -> String {
    AccountRecord::new(account(secret, "x"), 0).id
}

#[test]
fn adding_appends_new_accounts_and_skips_duplicates() {
    let data = with_accounts(&[("a", "A")], 10);
    let (updated, report) = add_accounts(&data, vec![account("a", "A again"), account("b", "B"), account("b", "B twice")], 20);
    assert_eq!(report, AddReport { added: 1, skipped_duplicates: 2 });
    assert_eq!(updated.accounts.len(), 2);
    assert_eq!(updated.accounts[0].account.name, "A");
}

#[test]
fn renaming_changes_the_labels_and_trims_them() {
    let data = with_accounts(&[("a", "A")], 10);
    let updated = rename_account(&data, &id_of("a"), &labels("  Work ", "  me@work.com "), 20).unwrap();
    assert_eq!(updated.accounts[0].account.issuer, "Work");
    assert_eq!(updated.accounts[0].account.name, "me@work.com");
    assert_eq!(updated.accounts[0].updated_at_ms, 20);
}

#[test]
fn renaming_keeps_the_secret_and_the_id() {
    let data = with_accounts(&[("a", "A")], 10);
    let updated = rename_account(&data, &id_of("a"), &labels("", "B"), 20).unwrap();
    assert_eq!(updated.accounts[0].account.secret, b"a");
    assert_eq!(updated.accounts[0].id, id_of("a"));
}

#[test]
fn an_account_cannot_be_renamed_to_nothing_or_found_by_a_wrong_id() {
    let data = with_accounts(&[("a", "A")], 10);
    assert_eq!(rename_account(&data, &id_of("a"), &labels("x", "   "), 20), Err(ChangeError::EmptyName));
    assert_eq!(rename_account(&data, "nope", &labels("x", "y"), 20), Err(ChangeError::AccountNotFound("nope".into())));
}

#[test]
fn deleting_removes_the_account_and_leaves_a_tombstone() {
    let data = with_accounts(&[("a", "A"), ("b", "B")], 10);
    let updated = delete_account(&data, &id_of("a"), 20).unwrap();
    assert_eq!(updated.accounts.len(), 1);
    assert_eq!(updated.tombstones, vec![Tombstone { id: id_of("a"), deleted_at_ms: 20 }]);
    assert_eq!(delete_account(&updated, &id_of("a"), 30), Err(ChangeError::AccountNotFound(id_of("a"))));
}

#[test]
fn a_deleted_account_can_be_added_again() {
    let data = with_accounts(&[("a", "A")], 10);
    let deleted = delete_account(&data, &id_of("a"), 20).unwrap();
    let (readded, report) = add_accounts(&deleted, vec![account("a", "A")], 30);
    assert_eq!(report.added, 1);
    assert!(readded.tombstones.is_empty());
}

#[test]
fn a_change_is_newer_than_anything_seen_even_if_this_clock_is_behind() {
    let data = with_accounts(&[("a", "A")], 1_000);
    let updated = rename_account(&data, &id_of("a"), &labels("", "B"), 5).unwrap();
    assert!(updated.accounts[0].updated_at_ms > 1_000);
}

#[test]
fn a_delete_on_one_device_and_a_rename_on_another_resolve_the_same_everywhere() {
    let original = with_accounts(&[("a", "A")], 10);
    let deleted = delete_account(&original, &id_of("a"), 20).unwrap();
    let renamed = rename_account(&original, &id_of("a"), &labels("", "B"), 15).unwrap();
    let one_way = merge(&deleted, &renamed);
    let other_way = merge(&renamed, &deleted);
    assert!(one_way.accounts.is_empty() && other_way.accounts.is_empty());
}
