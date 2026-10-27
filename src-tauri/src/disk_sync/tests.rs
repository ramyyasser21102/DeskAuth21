use super::*;
use crate::account_changes::AccountLabels;
use crate::state::AppState;
use crate::test_support::{device, device_with_account, remove_all, temp_directory, ACME_URI, GLOBEX_URI, PASSPHRASE};
use crate::vault_actions::{current_codes_now, delete_stored_account, import_accounts, rename_stored_account, unlock_vault};
use crate::vault_location::change_vault_folder;
use std::path::PathBuf;

/// Two unlocked devices sharing one vault file, as a sync service would leave them.
fn two_devices_sharing_a_vault() -> (AppState, AppState, PathBuf, Vec<PathBuf>) {
    let (first, first_dir) = device_with_account("sync-first", ACME_URI);
    let shared = temp_directory("sync-shared");
    change_vault_folder(&first, &shared).unwrap();
    let (second, second_dir) = device("sync-second");
    change_vault_folder(&second, &shared).unwrap();
    unlock_vault(&second, PASSPHRASE).unwrap();
    (first, second, shared.clone(), vec![first_dir, second_dir, shared])
}

fn sync(state: &AppState) -> DiskSyncOutcome {
    let mut vault = state.vault_guard();
    sync_from_disk(&state.vault_path(), &mut vault)
}

fn labels(state: &AppState) -> Vec<String> {
    current_codes_now(state).unwrap().into_iter().map(|view| view.label).collect()
}

#[test]
fn a_change_made_on_another_device_appears_without_unlocking_again() {
    let (first, second, _, dirs) = two_devices_sharing_a_vault();
    import_accounts(&first, GLOBEX_URI).unwrap();
    assert_eq!(sync(&second), DiskSyncOutcome::Updated);
    assert_eq!(labels(&second), vec!["Acme (a@b)", "Globex (c@d)"]);
    remove_all(&dirs);
}

#[test]
fn our_own_saves_are_not_mistaken_for_remote_changes() {
    let (first, _, _, dirs) = two_devices_sharing_a_vault();
    import_accounts(&first, GLOBEX_URI).unwrap();
    assert_eq!(sync(&first), DiskSyncOutcome::Unchanged);
    remove_all(&dirs);
}

#[test]
fn nothing_happens_when_the_file_has_not_changed() {
    let (_, second, _, dirs) = two_devices_sharing_a_vault();
    assert_eq!(sync(&second), DiskSyncOutcome::Unchanged);
    remove_all(&dirs);
}

#[test]
fn importing_on_a_stale_device_keeps_both_devices_additions() {
    let (first, second, _, dirs) = two_devices_sharing_a_vault();
    // The second device has not noticed anything yet, but the first one adds an account.
    let third_uri = "otpauth://totp/Initech:e@f?secret=ONSWG4TFOQ======&issuer=Initech";
    import_accounts(&first, GLOBEX_URI).unwrap();
    import_accounts(&second, third_uri).unwrap();
    assert_eq!(sync(&first), DiskSyncOutcome::Updated);
    assert_eq!(labels(&first), vec!["Acme (a@b)", "Globex (c@d)", "Initech (e@f)"]);
    assert_eq!(labels(&second).len(), 3);
    remove_all(&dirs);
}

#[test]
fn accounts_missing_from_a_restored_older_copy_are_written_back() {
    let (first, _, shared, dirs) = two_devices_sharing_a_vault();
    let vault_file = shared.join("vault.da21");
    let older_copy = std::fs::read(&vault_file).unwrap();
    import_accounts(&first, GLOBEX_URI).unwrap();
    // A sync conflict puts the older file back.
    std::fs::write(&vault_file, older_copy).unwrap();
    assert_eq!(sync(&first), DiskSyncOutcome::Updated);
    assert_eq!(labels(&first).len(), 2);
    let healed = first.vault_guard().session().unwrap().unseal(&std::fs::read(&vault_file).unwrap()).unwrap();
    assert_eq!(healed.accounts.len(), 2);
    remove_all(&dirs);
}

#[test]
fn a_different_vault_in_the_file_is_never_overwritten() {
    let (first, _, shared, dirs) = two_devices_sharing_a_vault();
    let (_, stranger_dir) = device_with_account("sync-stranger", GLOBEX_URI);
    let stranger_bytes = std::fs::read(stranger_dir.join("vault.da21")).unwrap();
    let vault_file = shared.join("vault.da21");
    std::fs::write(&vault_file, &stranger_bytes).unwrap();
    assert_eq!(sync(&first), DiskSyncOutcome::Unreadable);
    assert!(matches!(import_accounts(&first, ACME_URI), Err(crate::error::CommandError::VaultChangedElsewhere)));
    assert_eq!(std::fs::read(&vault_file).unwrap(), stranger_bytes);
    remove_all(&[dirs, vec![stranger_dir]].concat());
}

#[test]
fn a_locked_vault_or_a_missing_file_is_left_alone() {
    let (first, _, shared, dirs) = two_devices_sharing_a_vault();
    std::fs::remove_file(shared.join("vault.da21")).unwrap();
    assert_eq!(sync(&first), DiskSyncOutcome::Unchanged);
    crate::vault_actions::lock_vault(&first);
    assert_eq!(sync(&first), DiskSyncOutcome::Unchanged);
    remove_all(&dirs);
}

fn first_id(state: &AppState) -> String {
    current_codes_now(state).unwrap()[0].id.clone()
}

#[test]
fn a_delete_on_one_device_removes_the_account_on_the_other() {
    let (first, second, _, dirs) = two_devices_sharing_a_vault();
    delete_stored_account(&first, &first_id(&first)).unwrap();
    assert_eq!(sync(&second), DiskSyncOutcome::Updated);
    assert!(labels(&second).is_empty());
    remove_all(&dirs);
}

#[test]
fn a_rename_on_one_device_appears_on_the_other() {
    let (first, second, _, dirs) = two_devices_sharing_a_vault();
    rename_stored_account(&first, &first_id(&first), &AccountLabels { issuer: "Work".into(), name: "me@work.com".into() }).unwrap();
    assert_eq!(sync(&second), DiskSyncOutcome::Updated);
    assert_eq!(labels(&second), vec!["Work (me@work.com)"]);
    remove_all(&dirs);
}

#[test]
fn a_device_that_still_has_a_deleted_account_cannot_bring_it_back() {
    let (first, second, shared, dirs) = two_devices_sharing_a_vault();
    delete_stored_account(&first, &first_id(&first)).unwrap();
    // The second device never noticed; syncing must not resurrect the account.
    assert_eq!(sync(&second), DiskSyncOutcome::Updated);
    let on_disk = first.vault_guard().session().unwrap().unseal(&std::fs::read(shared.join("vault.da21")).unwrap()).unwrap();
    assert!(on_disk.accounts.is_empty());
    assert_eq!(on_disk.tombstones.len(), 1);
    remove_all(&dirs);
}

#[test]
fn a_rename_and_a_delete_made_apart_end_the_same_on_both_devices() {
    let (first, second, _, dirs) = two_devices_sharing_a_vault();
    let id = first_id(&first);
    // Both act on the same account before either has seen the other's change.
    rename_stored_account(&second, &id, &AccountLabels { issuer: "Work".into(), name: "renamed".into() }).unwrap();
    delete_stored_account(&first, &id).unwrap();
    sync(&second);
    sync(&first);
    assert_eq!(labels(&first), labels(&second));
    assert!(labels(&first).is_empty());
    remove_all(&dirs);
}

#[test]
fn the_folder_sync_picks_up_a_conflicted_copy_and_reports_an_update() {
    let (first, second, shared, dirs) = two_devices_sharing_a_vault();
    let main = shared.join("vault.da21");
    let older = std::fs::read(&main).unwrap();
    import_accounts(&first, GLOBEX_URI).unwrap();
    std::fs::write(shared.join("vault (conflicted copy).da21"), std::fs::read(&main).unwrap()).unwrap();
    std::fs::write(&main, older).unwrap();
    let outcome = sync_vault_folder(&second.vault_path(), &mut second.vault_guard());
    assert_eq!(outcome, DiskSyncOutcome::Updated);
    assert_eq!(labels(&second), vec!["Acme (a@b)", "Globex (c@d)"]);
    remove_all(&dirs);
}

#[test]
fn a_foreign_main_file_stops_the_folder_sync_before_it_touches_any_copies() {
    let (first, _, shared, dirs) = two_devices_sharing_a_vault();
    let (_, stranger_dir) = device_with_account("sync-stranger-2", GLOBEX_URI);
    std::fs::write(shared.join("vault.da21"), std::fs::read(stranger_dir.join("vault.da21")).unwrap()).unwrap();
    let outcome = sync_vault_folder(&first.vault_path(), &mut first.vault_guard());
    assert_eq!(outcome, DiskSyncOutcome::Unreadable);
    remove_all(&[dirs, vec![stranger_dir]].concat());
}
