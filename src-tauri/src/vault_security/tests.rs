use super::*;
use crate::conflict_copies::is_conflict_copy_name;
use crate::test_support::{device, remove_all, temp_directory, ACME_URI, PASSPHRASE};
use crate::vault_location::change_vault_folder;
use std::path::{Path, PathBuf};

/// A real version 1 vault with one account, written before unlock slots existed.
const V1_FIXTURE: &[u8] = include_bytes!("../vault/fixtures/v1_vault.da21");
const V1_PASSPHRASE: &str = "legacy fixture pass";

/// A device that has the old vault in a shared folder and has unlocked it.
fn device_with_v1_vault(label: &str, shared: &Path) -> (AppState, PathBuf) {
    let (state, directory) = device(label);
    change_vault_folder(&state, shared).unwrap();
    unlock_vault(&state, V1_PASSPHRASE).unwrap();
    (state, directory)
}

fn shared_folder_with_v1_vault(label: &str) -> PathBuf {
    let shared = temp_directory(label);
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(shared.join("vault.da21"), V1_FIXTURE).unwrap();
    shared
}

#[test]
fn a_new_vault_is_in_the_current_format_from_the_start() {
    let (state, directory) = crate::test_support::device_with_account("upgrade-new", ACME_URI);
    assert_eq!(vault_format_version(&state).unwrap(), 2);
    remove_all(&[directory]);
}

#[test]
fn upgrading_an_old_vault_keeps_a_copy_and_the_same_passphrase_and_data() {
    let shared = shared_folder_with_v1_vault("upgrade-copy");
    let (state, directory) = device_with_v1_vault("upgrade-copy-device", &shared);
    assert_eq!(vault_format_version(&state).unwrap(), 1);

    upgrade_vault_format(&state).unwrap();

    assert_eq!(vault_format_version(&state).unwrap(), 2);
    assert_eq!(std::fs::read(shared.join("vault.da21.v1.bak")).unwrap(), V1_FIXTURE);
    lock_vault(&state);
    unlock_vault(&state, V1_PASSPHRASE).unwrap();
    assert_eq!(vault_format_version(&state).unwrap(), 2);
    assert_eq!(current_codes_now(&state).unwrap().len(), 1);
    remove_all(&[directory, shared]);
}

#[test]
fn upgrading_twice_changes_nothing_the_second_time() {
    let shared = shared_folder_with_v1_vault("upgrade-twice");
    let (state, directory) = device_with_v1_vault("upgrade-twice-device", &shared);
    upgrade_vault_format(&state).unwrap();
    let after_first = std::fs::read(shared.join("vault.da21")).unwrap();
    upgrade_vault_format(&state).unwrap();
    assert_eq!(std::fs::read(shared.join("vault.da21")).unwrap(), after_first);
    remove_all(&[directory, shared]);
}

#[test]
fn upgrading_needs_an_unlocked_vault() {
    let shared = shared_folder_with_v1_vault("upgrade-locked");
    let (state, directory) = device_with_v1_vault("upgrade-locked-device", &shared);
    lock_vault(&state);
    assert!(matches!(upgrade_vault_format(&state), Err(CommandError::VaultLocked)));
    remove_all(&[directory, shared]);
}

#[test]
fn the_safety_copy_is_never_mistaken_for_a_conflicted_vault() {
    assert!(!is_conflict_copy_name("vault.da21.v1.bak", "vault.da21"));
}

#[test]
fn a_computer_still_on_the_old_format_is_stopped_not_overwritten_when_another_upgrades() {
    let shared = shared_folder_with_v1_vault("upgrade-two-devices");
    let (upgrader, first_dir) = device_with_v1_vault("upgrade-two-first", &shared);
    let (behind, second_dir) = device_with_v1_vault("upgrade-two-second", &shared);

    upgrade_vault_format(&upgrader).unwrap();
    let upgraded_file = std::fs::read(shared.join("vault.da21")).unwrap();

    let result = import_accounts(&behind, ACME_URI, None);
    assert!(matches!(result, Err(CommandError::VaultChangedElsewhere)));
    assert_eq!(std::fs::read(shared.join("vault.da21")).unwrap(), upgraded_file);

    lock_vault(&behind);
    unlock_vault(&behind, V1_PASSPHRASE).unwrap();
    assert_eq!(vault_format_version(&behind).unwrap(), 2);
    assert_eq!(import_accounts(&behind, ACME_URI, None).unwrap().added, 1);
    remove_all(&[first_dir, second_dir, shared]);
}

#[test]
fn a_new_vault_does_not_reuse_the_test_passphrase_by_accident() {
    assert_ne!(PASSPHRASE, V1_PASSPHRASE);
}
