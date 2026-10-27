use super::*;
use crate::account_changes::AccountLabels;
use crate::state::AppState;
use crate::test_support::{device, device_with_account, remove_all, temp_directory, ACME_URI, GLOBEX_URI, PASSPHRASE};
use crate::vault_actions::{current_codes_now, delete_stored_account, import_accounts, rename_stored_account, unlock_vault};
use crate::vault_location::change_vault_folder;

const MAIN_NAME: &str = "vault.da21";

#[test]
fn recognises_the_copies_that_sync_services_make() {
    for name in [
        "vault (Ann's conflicted copy 2026-10-27).da21",
        "vault (conflicted copy).da21",
        "vault.sync-conflict-20261027-101500-ABCDEFG.da21",
        "vault 2.da21",
        "vault (1).da21",
        "vault-LAPTOP.da21",
    ] {
        assert!(is_conflict_copy_name(name, MAIN_NAME), "{name} should count as a conflicted copy");
    }
}

#[test]
fn ignores_the_vault_itself_and_unrelated_or_already_handled_files() {
    for name in ["vault.da21", "vault.da21.merged", "vault.tmp", "settings.json", "other.da21", "notes vault.da21", "vault"] {
        assert!(!is_conflict_copy_name(name, MAIN_NAME), "{name} should not count");
    }
}

/// Two unlocked devices on one shared vault, plus the shared folder path.
fn two_devices() -> (AppState, AppState, std::path::PathBuf, Vec<std::path::PathBuf>) {
    let (first, first_dir) = device_with_account("copies-first", ACME_URI);
    let shared = temp_directory("copies-shared");
    change_vault_folder(&first, &shared).unwrap();
    let (second, second_dir) = device("copies-second");
    change_vault_folder(&second, &shared).unwrap();
    unlock_vault(&second, PASSPHRASE).unwrap();
    (first, second, shared.clone(), vec![first_dir, second_dir, shared])
}

/// What a sync conflict leaves: the main file holds one device's older state and
/// a conflicted copy holds the other device's newer one.
fn leave_conflict(writer: &AppState, shared: &std::path::Path, copy_name: &str, change: impl FnOnce(&AppState)) {
    let main = shared.join(MAIN_NAME);
    let older = std::fs::read(&main).unwrap();
    change(writer);
    std::fs::write(shared.join(copy_name), std::fs::read(&main).unwrap()).unwrap();
    std::fs::write(&main, older).unwrap();
}

fn merge_for(state: &AppState) -> usize {
    let mut vault = state.vault_guard();
    merge_conflict_copies(&state.vault_path(), &mut vault)
}

fn labels(state: &AppState) -> Vec<String> {
    current_codes_now(state).unwrap().into_iter().map(|view| view.label).collect()
}

#[test]
fn an_account_added_on_the_other_device_is_recovered_from_its_copy() {
    let (first, second, shared, dirs) = two_devices();
    leave_conflict(&second, &shared, "vault (conflicted copy).da21", |device| {
        import_accounts(device, GLOBEX_URI).unwrap();
    });
    assert_eq!(merge_for(&first), 1);
    assert_eq!(labels(&first), vec!["Acme (a@b)", "Globex (c@d)"]);
    let saved = first.vault_guard().session().unwrap().unseal(&std::fs::read(shared.join(MAIN_NAME)).unwrap()).unwrap();
    assert_eq!(saved.accounts.len(), 2);
    remove_all(&dirs);
}

#[test]
fn a_merged_copy_is_set_aside_not_deleted() {
    let (first, second, shared, dirs) = two_devices();
    leave_conflict(&second, &shared, "vault (conflicted copy).da21", |device| {
        import_accounts(device, GLOBEX_URI).unwrap();
    });
    merge_for(&first);
    assert!(!shared.join("vault (conflicted copy).da21").exists());
    assert!(shared.join("vault (conflicted copy).da21.merged").exists());
    assert_eq!(merge_for(&first), 0);
    remove_all(&dirs);
}

#[test]
fn a_delete_and_a_rename_made_in_a_conflicting_copy_are_applied() {
    let (first, second, shared, dirs) = two_devices();
    import_accounts(&first, GLOBEX_URI).unwrap();
    crate::disk_sync::sync_from_disk(&second.vault_path(), &mut second.vault_guard());
    let acme = current_codes_now(&second).unwrap()[0].id.clone();
    leave_conflict(&second, &shared, "vault.sync-conflict-20261027-101500-ABCDEFG.da21", |device| {
        rename_stored_account(device, &current_codes_now(device).unwrap()[1].id, &AccountLabels { issuer: "Globex".into(), name: "renamed".into() }).unwrap();
        delete_stored_account(device, &acme).unwrap();
    });
    assert_eq!(merge_for(&first), 1);
    assert_eq!(labels(&first), vec!["Globex (renamed)"]);
    remove_all(&dirs);
}

#[test]
fn a_copy_with_nothing_new_is_still_set_aside_without_rewriting_the_vault() {
    let (first, _, shared, dirs) = two_devices();
    let main = shared.join(MAIN_NAME);
    std::fs::copy(&main, shared.join("vault 2.da21")).unwrap();
    let before = std::fs::read(&main).unwrap();
    assert_eq!(merge_for(&first), 1);
    assert_eq!(std::fs::read(&main).unwrap(), before);
    assert!(shared.join("vault 2.da21.merged").exists());
    remove_all(&dirs);
}

#[test]
fn a_file_from_a_different_vault_is_left_exactly_as_it_is() {
    let (first, _, shared, dirs) = two_devices();
    let (_, stranger_dir) = device_with_account("copies-stranger", GLOBEX_URI);
    let stranger_bytes = std::fs::read(stranger_dir.join(MAIN_NAME)).unwrap();
    std::fs::write(shared.join("vault 2.da21"), &stranger_bytes).unwrap();
    assert_eq!(merge_for(&first), 0);
    assert_eq!(merge_for(&first), 0);
    assert_eq!(std::fs::read(shared.join("vault 2.da21")).unwrap(), stranger_bytes);
    assert_eq!(labels(&first), vec!["Acme (a@b)"]);
    remove_all(&[dirs, vec![stranger_dir]].concat());
}

#[test]
fn nothing_is_merged_while_locked_or_when_there_are_no_copies() {
    let (first, _, _, dirs) = two_devices();
    assert_eq!(merge_for(&first), 0);
    crate::vault_actions::lock_vault(&first);
    assert_eq!(merge_for(&first), 0);
    remove_all(&dirs);
}

#[test]
fn a_copy_that_cannot_be_set_aside_is_merged_once_and_not_retried() {
    let (first, second, shared, dirs) = two_devices();
    leave_conflict(&second, &shared, "vault 2.da21", |device| {
        import_accounts(device, GLOBEX_URI).unwrap();
    });
    // A directory already sits where the ".merged" file would go, so renaming fails.
    std::fs::create_dir(shared.join("vault 2.da21.merged")).unwrap();
    assert_eq!(merge_for(&first), 1);
    assert!(shared.join("vault 2.da21").exists());
    assert_eq!(merge_for(&first), 0);
    assert_eq!(labels(&first).len(), 2);
    remove_all(&dirs);
}
