use super::*;
use crate::error::CommandError;
use crate::test_support::{device, device_with_account, remove_all, temp_directory, ACME_URI, FAST_KDF, GLOBEX_URI, PASSPHRASE};
use crate::vault::VaultError;
use crate::vault_actions::{current_codes_now, import_accounts, lock_vault, unlock_vault};
use crate::vault_location::change_vault_folder;

const NEW_PASSPHRASE: &str = "a brand new passphrase";

#[test]
fn a_changed_passphrase_opens_the_vault_and_the_old_one_no_longer_does() {
    let (state, directory) = device_with_account("pw-change", ACME_URI);
    change_passphrase(&state, PASSPHRASE, NEW_PASSPHRASE, FAST_KDF).unwrap();
    lock_vault(&state);
    assert!(matches!(unlock_vault(&state, PASSPHRASE), Err(CommandError::Vault(VaultError::Decrypt))));
    unlock_vault(&state, NEW_PASSPHRASE).unwrap();
    assert_eq!(current_codes_now(&state).unwrap().len(), 1);
    remove_all(&[directory]);
}

#[test]
fn changing_the_passphrase_needs_the_current_one_even_though_the_vault_is_open() {
    let (state, directory) = device_with_account("pw-current", ACME_URI);
    let result = change_passphrase(&state, "not the passphrase", NEW_PASSPHRASE, FAST_KDF);
    assert!(matches!(result, Err(CommandError::Vault(VaultError::Decrypt))));
    lock_vault(&state);
    unlock_vault(&state, PASSPHRASE).unwrap();
    remove_all(&[directory]);
}

#[test]
fn a_new_passphrase_that_is_too_short_changes_nothing() {
    let (state, directory) = device_with_account("pw-short", ACME_URI);
    assert!(matches!(change_passphrase(&state, PASSPHRASE, "short", FAST_KDF), Err(CommandError::PassphraseTooShort(_))));
    lock_vault(&state);
    unlock_vault(&state, PASSPHRASE).unwrap();
    remove_all(&[directory]);
}

#[test]
fn an_older_format_vault_must_be_upgraded_before_the_passphrase_can_change() {
    let shared = temp_directory("pw-legacy");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(shared.join("vault.da21"), include_bytes!("../vault/fixtures/v1_vault.da21")).unwrap();
    let (state, directory) = device("pw-legacy-device");
    change_vault_folder(&state, &shared).unwrap();
    unlock_vault(&state, "legacy fixture pass").unwrap();
    let result = change_passphrase(&state, "legacy fixture pass", NEW_PASSPHRASE, FAST_KDF);
    assert!(matches!(result, Err(CommandError::Vault(VaultError::NeedsUpgrade))));
    remove_all(&[directory, shared]);
}

/// Two unlocked computers sharing one vault file.
fn two_computers(label: &str) -> (AppState, AppState, Vec<std::path::PathBuf>) {
    let (first, first_dir) = device_with_account(&format!("{label}-first"), ACME_URI);
    let shared = temp_directory(&format!("{label}-shared"));
    change_vault_folder(&first, &shared).unwrap();
    let (second, second_dir) = device(&format!("{label}-second"));
    change_vault_folder(&second, &shared).unwrap();
    unlock_vault(&second, PASSPHRASE).unwrap();
    (first, second, vec![first_dir, second_dir, shared])
}

#[test]
fn another_computer_saving_does_not_undo_a_passphrase_change() {
    let (first, second, directories) = two_computers("pw-sync");
    change_passphrase(&first, PASSPHRASE, NEW_PASSPHRASE, FAST_KDF).unwrap();

    // The second computer still holds the old slots, and saves a change of its own.
    import_accounts(&second, GLOBEX_URI, None).unwrap();

    lock_vault(&first);
    unlock_vault(&first, NEW_PASSPHRASE).unwrap();
    assert!(unlock_vault(&first, PASSPHRASE).is_err());
    assert_eq!(current_codes_now(&first).unwrap().len(), 2);
    remove_all(&directories);
}

#[test]
fn another_computer_saving_does_not_drop_a_recovery_key() {
    let (first, second, directories) = two_computers("rk-sync");
    let phrase = generate_recovery_key(&first).unwrap();
    save_recovery_key(&first, &phrase, PASSPHRASE).unwrap();

    import_accounts(&second, GLOBEX_URI, None).unwrap();

    assert!(has_recovery_key(&second).unwrap(), "the second computer follows the file's slots");
    lock_vault(&first);
    reset_passphrase_with_recovery_key(&first, &phrase, NEW_PASSPHRASE, FAST_KDF).unwrap();
    remove_all(&directories);
}
