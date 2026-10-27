use super::*;
use crate::key_device::{KeyDevice, KeyError, KeyRegistration, NoKeySupport};
use crate::recovery_key;
use crate::test_support::FakeKey;
use crate::vault::layout::parse;
use crate::vault::slot::Slot;

const PASSPHRASE: &str = "correct horse battery";
const NEW_PASSPHRASE: &str = "a brand new passphrase";

/// An open vault with a recovery key, as a vault must have before a hardware key is required.
fn vault_with_recovery_key() -> (Session, String) {
    let mut session = Session::create(PASSPHRASE, FAST_PARAMS).unwrap();
    let phrase = recovery_key::generate().unwrap().to_string();
    session.set_recovery_key(&recovery_key::normalize(&phrase).unwrap(), PASSPHRASE).unwrap();
    (session, phrase)
}

/// A vault that requires `key`, and the file it was saved to.
fn protected_vault(key: &FakeKey) -> (Session, Vec<u8>) {
    let (mut session, _) = vault_with_recovery_key();
    session.enable_hardware_key(&reg(key, None, "Desk key"), PASSPHRASE, FAST_PARAMS).unwrap();
    let file = session.seal(&sample_data()).unwrap();
    (session, file)
}

fn reg<'a>(device: &'a dyn KeyDevice, pin: Option<&'a str>, label: &'a str) -> KeyRegistration<'a> {
    KeyRegistration { device, pin, label }
}

fn open(file: &[u8], passphrase: &str, key: &FakeKey) -> Result<Session, VaultError> {
    Session::open_with_key(file, passphrase, key, None).map(|(session, _)| session)
}

fn slot_kinds(file: &[u8]) -> Vec<u8> {
    parse(file).unwrap().slots.iter().map(Slot::kind).collect()
}

#[test]
fn a_recovery_key_must_exist_before_a_hardware_key_can_be_required() {
    let mut session = Session::create(PASSPHRASE, FAST_PARAMS).unwrap();
    let key = FakeKey::new(1);
    let result = session.enable_hardware_key(&reg(&key, None, "Desk key"), PASSPHRASE, FAST_PARAMS);
    assert!(matches!(result, Err(VaultError::NeedsRecoveryKey)));
    assert_eq!(key.registrations.get(), 0, "the key was not touched");
}

#[test]
fn enabling_needs_the_current_passphrase_and_does_not_touch_the_key_without_it() {
    let (mut session, _) = vault_with_recovery_key();
    let key = FakeKey::new(1);
    let result = session.enable_hardware_key(&reg(&key, None, "Desk key"), "not the passphrase", FAST_PARAMS);
    assert!(matches!(result, Err(VaultError::Decrypt)));
    assert_eq!(key.registrations.get(), 0);
}

#[test]
fn once_required_the_passphrase_alone_no_longer_opens_the_vault() {
    let key = FakeKey::new(1);
    let (_, file) = protected_vault(&key);
    assert!(!slot_kinds(&file).contains(&1), "no slot opens the vault by passphrase alone");
    assert!(matches!(Session::open(&file, PASSPHRASE), Err(VaultError::Key(KeyError::Unsupported))));
    key.is_plugged_in.set(false);
    assert!(matches!(open(&file, PASSPHRASE, &key), Err(VaultError::Key(KeyError::NoKey))));
}

#[test]
fn the_passphrase_and_the_key_together_open_the_vault_with_its_data() {
    let key = FakeKey::new(1);
    let (_, file) = protected_vault(&key);
    let (session, data) = Session::open_with_key(&file, PASSPHRASE, &key, None).unwrap();
    assert_eq!(data, sample_data());
    assert!(session.requires_hardware_key());
    assert_eq!(session.hardware_keys().len(), 1);
}

#[test]
fn a_wrong_passphrase_is_refused_before_the_key_is_asked_for_a_touch() {
    let key = FakeKey::new(1);
    let (_, file) = protected_vault(&key);
    let touches_before = key.derivations.get();
    assert!(matches!(open(&file, "wrong passphrase", &key), Err(VaultError::Decrypt)));
    assert_eq!(key.derivations.get(), touches_before);
}

#[test]
fn a_different_key_is_refused_with_a_clear_reason() {
    let (_, file) = protected_vault(&FakeKey::new(1));
    assert!(matches!(open(&file, PASSPHRASE, &FakeKey::new(2)), Err(VaultError::Key(KeyError::NotThisKey))));
}

#[test]
fn a_key_with_a_pin_needs_it_for_registering_and_unlocking() {
    let key = FakeKey::with_pin(4, "1234");
    let (mut session, _) = vault_with_recovery_key();
    let refused = session.enable_hardware_key(&reg(&key, None, "Pin key"), PASSPHRASE, FAST_PARAMS);
    assert!(matches!(refused, Err(VaultError::Key(KeyError::PinProblem))));
    session.enable_hardware_key(&reg(&key, Some("1234"), "Pin key"), PASSPHRASE, FAST_PARAMS).unwrap();
    let file = session.seal(&sample_data()).unwrap();
    assert!(Session::open_with_key(&file, PASSPHRASE, &key, Some("1234")).is_ok());
    assert!(matches!(Session::open_with_key(&file, PASSPHRASE, &key, Some("0000")), Err(VaultError::Key(KeyError::PinProblem))));
}

#[test]
fn a_second_key_opens_the_vault_just_as_well() {
    let (first, second) = (FakeKey::new(1), FakeKey::new(2));
    let (mut session, _) = protected_vault(&first);
    session.add_hardware_key(&reg(&second, None, "Spare")).unwrap();
    let file = session.seal(&sample_data()).unwrap();
    assert!(open(&file, PASSPHRASE, &first).is_ok());
    assert!(open(&file, PASSPHRASE, &second).is_ok());
    assert_eq!(session.hardware_keys().len(), 2);
}

#[test]
fn a_removed_key_stops_working_and_the_last_key_cannot_be_removed() {
    let (first, second) = (FakeKey::new(1), FakeKey::new(2));
    let (mut session, _) = protected_vault(&first);
    session.add_hardware_key(&reg(&second, None, "Spare")).unwrap();
    let first_id = session.hardware_keys()[0].credential_id.clone();
    session.remove_hardware_key(&first_id).unwrap();
    let file = session.seal(&sample_data()).unwrap();
    assert!(matches!(open(&file, PASSPHRASE, &first), Err(VaultError::Key(KeyError::NotThisKey))));
    assert!(open(&file, PASSPHRASE, &second).is_ok());
    let last_id = session.hardware_keys()[0].credential_id.clone();
    assert!(matches!(session.remove_hardware_key(&last_id), Err(VaultError::LastHardwareKey)));
    assert!(matches!(session.remove_hardware_key(&[0xff]), Err(VaultError::UnknownHardwareKey)));
}

#[test]
fn turning_the_requirement_off_needs_the_passphrase_and_restores_passphrase_only_unlocking() {
    let key = FakeKey::new(1);
    let (mut session, _) = protected_vault(&key);
    assert!(matches!(session.disable_hardware_keys("wrong", FAST_PARAMS), Err(VaultError::Decrypt)));
    session.disable_hardware_keys(PASSPHRASE, FAST_PARAMS).unwrap();
    let file = session.seal(&sample_data()).unwrap();
    assert!(Session::open(&file, PASSPHRASE).is_ok());
    assert!(!session.requires_hardware_key());
}

#[test]
fn changing_the_passphrase_keeps_the_key_requirement_and_the_registered_keys() {
    let key = FakeKey::new(1);
    let (mut session, _) = protected_vault(&key);
    session.change_passphrase(PASSPHRASE, NEW_PASSPHRASE, FAST_PARAMS).unwrap();
    let file = session.seal(&sample_data()).unwrap();
    assert!(matches!(open(&file, PASSPHRASE, &key), Err(VaultError::Decrypt)));
    assert!(open(&file, NEW_PASSPHRASE, &key).is_ok());
    assert!(matches!(Session::open(&file, NEW_PASSPHRASE), Err(VaultError::Key(KeyError::Unsupported))));
}

#[test]
fn the_recovery_key_opens_the_vault_and_resetting_the_passphrase_drops_the_key_requirement() {
    let key = FakeKey::new(1);
    let (mut session, phrase) = vault_with_recovery_key();
    session.enable_hardware_key(&reg(&key, None, "Desk key"), PASSPHRASE, FAST_PARAMS).unwrap();
    let file = session.seal(&sample_data()).unwrap();
    let normalized = recovery_key::normalize(&phrase).unwrap();
    let (mut recovered, data) = Session::open_with_recovery(&file, &normalized).unwrap();
    assert_eq!(data, sample_data());
    recovered.set_passphrase(NEW_PASSPHRASE, FAST_PARAMS).unwrap();
    assert!(!recovered.requires_hardware_key());
    assert!(Session::open(&recovered.seal(&data).unwrap(), NEW_PASSPHRASE).is_ok());
}

#[test]
fn editing_any_byte_of_a_hardware_vaults_header_makes_it_fail_to_open() {
    let key = FakeKey::new(1);
    let (_, file) = protected_vault(&key);
    let header_len = file.len() - parse(&file).unwrap().ciphertext.len() - 24;
    for position in 0..header_len {
        let mut edited = file.clone();
        edited[position] ^= 1;
        assert!(open(&edited, PASSPHRASE, &key).is_err(), "byte {position}");
    }
}

#[test]
fn changes_made_on_another_computer_that_keep_the_arrangement_are_followed() {
    let key = FakeKey::new(1);
    let (mut ours, file) = protected_vault(&key);
    let (mut theirs, _) = Session::open_with_key(&file, PASSPHRASE, &key, None).unwrap();
    theirs.change_passphrase(PASSPHRASE, NEW_PASSPHRASE, FAST_PARAMS).unwrap();
    ours.adopt_slots_from(&theirs.seal(&sample_data()).unwrap()).unwrap();
    ours.add_hardware_key(&reg(&FakeKey::new(2), None, "Spare")).unwrap();
}

#[test]
fn a_vault_turned_off_and_on_again_elsewhere_makes_the_held_secrets_unusable() {
    let key = FakeKey::new(1);
    let (mut ours, file) = protected_vault(&key);
    let (mut theirs, _) = Session::open_with_key(&file, PASSPHRASE, &key, None).unwrap();
    theirs.disable_hardware_keys(PASSPHRASE, FAST_PARAMS).unwrap();
    theirs.enable_hardware_key(&reg(&key, None, "Desk key"), PASSPHRASE, FAST_PARAMS).unwrap();
    ours.adopt_slots_from(&theirs.seal(&sample_data()).unwrap()).unwrap();
    assert!(matches!(ours.add_hardware_key(&reg(&FakeKey::new(2), None, "Spare")), Err(VaultError::LockAndUnlockFirst)));
    assert!(matches!(ours.change_passphrase(PASSPHRASE, NEW_PASSPHRASE, FAST_PARAMS), Err(VaultError::LockAndUnlockFirst)));
}

#[test]
fn no_key_support_refuses_clearly() {
    let (mut session, _) = vault_with_recovery_key();
    let result = session.enable_hardware_key(&reg(&NoKeySupport, None, "Desk key"), PASSPHRASE, FAST_PARAMS);
    assert!(matches!(result, Err(VaultError::Key(KeyError::Unsupported))));
}
