use super::*;
use crate::error::CommandError;
use crate::key_device::{KeyError, KeyRegistration};
use crate::test_support::{device, device_with_account, remove_all, temp_directory, FakeKey, ACME_URI, FAST_KDF, GLOBEX_URI, PASSPHRASE};
use crate::vault::VaultError;
use crate::vault_actions::{current_codes_now, import_accounts, lock_vault, unlock_vault, unlock_vault_with_key};
use crate::vault_location::change_vault_folder;

fn registration<'a>(key: &'a FakeKey, label: &'a str) -> KeyRegistration<'a> {
    KeyRegistration { device: key, pin: None, label }
}

/// An open vault with one account and a recovery key, ready for a hardware key.
fn vault_ready_for_a_key(label: &str) -> (AppState, std::path::PathBuf) {
    let (state, directory) = device_with_account(label, ACME_URI);
    let phrase = generate_recovery_key(&state).unwrap();
    save_recovery_key(&state, &phrase, PASSPHRASE).unwrap();
    (state, directory)
}

fn enable(state: &AppState, key: &FakeKey) {
    change_session_then_save(state, |session| session.enable_hardware_key(&registration(key, "Desk key"), PASSPHRASE, FAST_KDF)).unwrap();
}

#[test]
fn a_hardware_key_cannot_be_required_before_a_recovery_key_exists() {
    let (state, directory) = device_with_account("hw-norecovery", ACME_URI);
    let result = enable_hardware_key(&state, &registration(&FakeKey::new(1), "Desk key"), PASSPHRASE);
    assert!(matches!(result, Err(CommandError::Vault(VaultError::NeedsRecoveryKey))));
    remove_all(&[directory]);
}

#[test]
fn once_required_unlocking_needs_the_passphrase_and_the_key_and_keeps_the_accounts() {
    let (state, directory) = vault_ready_for_a_key("hw-cycle");
    let key = FakeKey::new(1);
    enable(&state, &key);
    lock_vault(&state);

    assert!(unlock_needs_hardware_key(&state).unwrap());
    assert!(matches!(unlock_vault(&state, PASSPHRASE), Err(CommandError::Vault(VaultError::Key(KeyError::Unsupported)))));
    key.is_plugged_in.set(false);
    assert!(matches!(unlock_vault_with_key(&state, PASSPHRASE, &key, None), Err(CommandError::Vault(VaultError::Key(KeyError::NoKey)))));
    key.is_plugged_in.set(true);
    unlock_vault_with_key(&state, PASSPHRASE, &key, None).unwrap();
    assert_eq!(current_codes_now(&state).unwrap().len(), 1);
    remove_all(&[directory]);
}

#[test]
fn a_wrong_passphrase_is_refused_without_asking_the_key_for_a_touch() {
    let (state, directory) = vault_ready_for_a_key("hw-wrongpass");
    let key = FakeKey::new(1);
    enable(&state, &key);
    lock_vault(&state);
    let touches = key.derivations.get();
    assert!(matches!(unlock_vault_with_key(&state, "not the passphrase", &key, None), Err(CommandError::Vault(VaultError::Decrypt))));
    assert_eq!(key.derivations.get(), touches);
    remove_all(&[directory]);
}

#[test]
fn keys_are_listed_and_a_second_one_can_be_added() {
    let (state, directory) = vault_ready_for_a_key("hw-list");
    enable(&state, &FakeKey::new(1));
    add_hardware_key(&state, &registration(&FakeKey::new(2), "Spare")).unwrap();
    let (is_required, keys) = hardware_keys(&state).unwrap();
    assert!(is_required);
    assert_eq!(keys.iter().map(|key| key.label.as_str()).collect::<Vec<_>>(), vec!["Desk key", "Spare"]);
    remove_all(&[directory]);
}

#[test]
fn a_removed_key_stops_working_and_the_last_one_cannot_be_removed() {
    let (state, directory) = vault_ready_for_a_key("hw-remove");
    let (first, second) = (FakeKey::new(1), FakeKey::new(2));
    enable(&state, &first);
    add_hardware_key(&state, &registration(&second, "Spare")).unwrap();
    let first_id = to_hex(&hardware_keys(&state).unwrap().1[0].credential_id);
    remove_hardware_key(&state, &first_id).unwrap();
    let last = to_hex(&hardware_keys(&state).unwrap().1[0].credential_id);
    assert!(matches!(remove_hardware_key(&state, &last), Err(CommandError::Vault(VaultError::LastHardwareKey))));
    lock_vault(&state);
    assert!(matches!(unlock_vault_with_key(&state, PASSPHRASE, &first, None), Err(CommandError::Vault(VaultError::Key(KeyError::NotThisKey)))));
    unlock_vault_with_key(&state, PASSPHRASE, &second, None).unwrap();
    remove_all(&[directory]);
}

#[test]
fn removing_a_key_that_is_not_registered_is_refused() {
    let (state, directory) = vault_ready_for_a_key("hw-unknown");
    enable(&state, &FakeKey::new(1));
    assert!(matches!(remove_hardware_key(&state, "zz"), Err(CommandError::Vault(VaultError::UnknownHardwareKey))));
    assert!(matches!(remove_hardware_key(&state, "ffff"), Err(CommandError::Vault(VaultError::UnknownHardwareKey))));
    remove_all(&[directory]);
}

#[test]
fn turning_the_requirement_off_needs_the_passphrase_and_restores_passphrase_only_unlocking() {
    let (state, directory) = vault_ready_for_a_key("hw-off");
    enable(&state, &FakeKey::new(1));
    assert!(matches!(disable_hardware_keys(&state, "wrong"), Err(CommandError::Vault(VaultError::Decrypt))));
    disable_hardware_keys(&state, PASSPHRASE).unwrap();
    lock_vault(&state);
    assert!(!unlock_needs_hardware_key(&state).unwrap());
    unlock_vault(&state, PASSPHRASE).unwrap();
    remove_all(&[directory]);
}

#[test]
fn a_computer_saving_after_another_requires_a_key_keeps_the_requirement() {
    let (first, first_dir) = vault_ready_for_a_key("hw-sync-first");
    let shared = temp_directory("hw-sync-shared");
    change_vault_folder(&first, &shared).unwrap();
    let (second, second_dir) = device("hw-sync-second");
    change_vault_folder(&second, &shared).unwrap();
    unlock_vault(&second, PASSPHRASE).unwrap();

    let key = FakeKey::new(1);
    enable(&first, &key);
    import_accounts(&second, GLOBEX_URI, None).unwrap();

    assert!(hardware_keys(&second).unwrap().0, "the second computer follows the file");
    lock_vault(&first);
    unlock_vault_with_key(&first, PASSPHRASE, &key, None).unwrap();
    assert_eq!(current_codes_now(&first).unwrap().len(), 2);
    assert!(unlock_vault(&first, PASSPHRASE).is_err());
    remove_all(&[first_dir, second_dir, shared]);
}

#[test]
fn credential_ids_round_trip_through_hex_and_bad_text_is_refused() {
    let bytes = vec![0, 1, 0xab, 0xff];
    assert_eq!(from_hex(&to_hex(&bytes)), Some(bytes));
    assert_eq!(to_hex(&[]), "");
    for bad in ["abc", "zz", "é1"] {
        assert_eq!(from_hex(bad), None, "{bad}");
    }
}
