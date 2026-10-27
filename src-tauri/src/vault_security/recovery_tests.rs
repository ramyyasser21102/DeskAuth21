use super::*;
use crate::error::CommandError;
use crate::recovery_key::RecoveryKeyError;
use crate::test_support::{device_with_account, remove_all, ACME_URI, FAST_KDF, PASSPHRASE};
use crate::vault::VaultError;
use crate::vault_actions::{current_codes_now, lock_vault, unlock_vault};

const NEW_PASSPHRASE: &str = "a brand new passphrase";

#[test]
fn a_new_vault_has_no_recovery_key_until_one_is_saved() {
    let (state, directory) = device_with_account("rk-none", ACME_URI);
    assert!(!has_recovery_key(&state).unwrap());
    let phrase = generate_recovery_key(&state).unwrap();
    assert_eq!(phrase.split(' ').count(), 24);
    assert!(!has_recovery_key(&state).unwrap(), "generating alone saves nothing");
    save_recovery_key(&state, &phrase, PASSPHRASE).unwrap();
    assert!(has_recovery_key(&state).unwrap());
    remove_all(&[directory]);
}

#[test]
fn saving_a_recovery_key_needs_the_current_passphrase() {
    let (state, directory) = device_with_account("rk-passphrase", ACME_URI);
    let phrase = generate_recovery_key(&state).unwrap();
    let result = save_recovery_key(&state, &phrase, "not the passphrase");
    assert!(matches!(result, Err(CommandError::Vault(VaultError::Decrypt))));
    assert!(!has_recovery_key(&state).unwrap());
    remove_all(&[directory]);
}

#[test]
fn the_recovery_key_opens_the_vault_and_sets_a_new_passphrase() {
    let (state, directory) = device_with_account("rk-reset", ACME_URI);
    let phrase = generate_recovery_key(&state).unwrap();
    save_recovery_key(&state, &phrase, PASSPHRASE).unwrap();
    lock_vault(&state);

    reset_passphrase_with_recovery_key(&state, &phrase, NEW_PASSPHRASE, FAST_KDF).unwrap();
    assert_eq!(current_codes_now(&state).unwrap().len(), 1, "the vault is open");

    lock_vault(&state);
    assert!(unlock_vault(&state, PASSPHRASE).is_err());
    unlock_vault(&state, NEW_PASSPHRASE).unwrap();
    assert!(has_recovery_key(&state).unwrap(), "the recovery key keeps working");
    remove_all(&[directory]);
}

#[test]
fn a_recovery_key_typed_in_capitals_with_odd_spacing_still_works() {
    let (state, directory) = device_with_account("rk-messy", ACME_URI);
    let phrase = generate_recovery_key(&state).unwrap();
    save_recovery_key(&state, &phrase, PASSPHRASE).unwrap();
    lock_vault(&state);
    let messy = format!("  {}  ", phrase.to_uppercase().replace(' ', " \n "));
    reset_passphrase_with_recovery_key(&state, &messy, NEW_PASSPHRASE, FAST_KDF).unwrap();
    remove_all(&[directory]);
}

#[test]
fn a_wrong_recovery_key_is_refused_and_the_vault_stays_locked() {
    let (state, directory) = device_with_account("rk-wrong", ACME_URI);
    let phrase = generate_recovery_key(&state).unwrap();
    save_recovery_key(&state, &phrase, PASSPHRASE).unwrap();
    lock_vault(&state);

    let other = generate_recovery_key_for_tests();
    let result = reset_passphrase_with_recovery_key(&state, &other, NEW_PASSPHRASE, FAST_KDF);
    assert!(matches!(result, Err(CommandError::Vault(VaultError::WrongRecoveryKey))));
    assert!(matches!(current_codes_now(&state), Err(CommandError::VaultLocked)));
    unlock_vault(&state, PASSPHRASE).unwrap();
    remove_all(&[directory]);
}

fn generate_recovery_key_for_tests() -> String {
    crate::recovery_key::generate().unwrap().to_string()
}

#[test]
fn a_misspelt_or_short_recovery_key_says_what_is_wrong() {
    let (state, directory) = device_with_account("rk-typo", ACME_URI);
    lock_vault(&state);
    let short = reset_passphrase_with_recovery_key(&state, "one two three", NEW_PASSPHRASE, FAST_KDF);
    assert!(matches!(short, Err(CommandError::RecoveryKey(RecoveryKeyError::WrongWordCount(3)))));
    let words = vec!["cabage"; 24].join(" ");
    let unknown = reset_passphrase_with_recovery_key(&state, &words, NEW_PASSPHRASE, FAST_KDF);
    assert!(matches!(unknown, Err(CommandError::RecoveryKey(RecoveryKeyError::UnknownWord(_)))));
    remove_all(&[directory]);
}

#[test]
fn saving_a_new_recovery_key_replaces_the_old_one() {
    let (state, directory) = device_with_account("rk-replace", ACME_URI);
    let first = generate_recovery_key(&state).unwrap();
    save_recovery_key(&state, &first, PASSPHRASE).unwrap();
    let second = generate_recovery_key(&state).unwrap();
    save_recovery_key(&state, &second, PASSPHRASE).unwrap();
    lock_vault(&state);
    let old = reset_passphrase_with_recovery_key(&state, &first, NEW_PASSPHRASE, FAST_KDF);
    assert!(matches!(old, Err(CommandError::Vault(VaultError::WrongRecoveryKey))));
    reset_passphrase_with_recovery_key(&state, &second, NEW_PASSPHRASE, FAST_KDF).unwrap();
    remove_all(&[directory]);
}

#[test]
fn resetting_with_a_too_short_new_passphrase_changes_nothing() {
    let (state, directory) = device_with_account("rk-short", ACME_URI);
    let phrase = generate_recovery_key(&state).unwrap();
    save_recovery_key(&state, &phrase, PASSPHRASE).unwrap();
    lock_vault(&state);
    let result = reset_passphrase_with_recovery_key(&state, &phrase, "short", FAST_KDF);
    assert!(matches!(result, Err(CommandError::PassphraseTooShort(_))));
    unlock_vault(&state, PASSPHRASE).unwrap();
    remove_all(&[directory]);
}
