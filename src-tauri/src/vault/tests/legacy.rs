use super::*;
use crate::vault::crypto::{derive_key, encrypt};
use crate::vault::header::{associated_data, Header, NONCE_LEN, SALT_LEN};
use crate::vault::layout::{file_version, VERSION_1, VERSION_2};
use crate::vault::legacy::LegacyKeys;

#[test]
fn a_real_version_1_vault_still_opens_with_its_passphrase() {
    let (session, data) = Session::open(V1_FIXTURE, V1_FIXTURE_PASSPHRASE).unwrap();
    assert_eq!(data, sample_data());
    assert_eq!(session.format_version(), 1);
    assert_eq!(file_version(V1_FIXTURE), Some(VERSION_1));
}

#[test]
fn a_version_1_vault_rejects_the_wrong_passphrase() {
    assert!(matches!(Session::open(V1_FIXTURE, "wrong"), Err(VaultError::Decrypt)));
}

#[test]
fn a_version_1_vault_keeps_saving_in_version_1_until_upgraded() {
    let (session, data) = Session::open(V1_FIXTURE, V1_FIXTURE_PASSPHRASE).unwrap();
    let saved = session.seal(&data).unwrap();
    assert_eq!(file_version(&saved), Some(VERSION_1));
    assert_eq!(Session::open(&saved, V1_FIXTURE_PASSPHRASE).unwrap().1, data);
}

#[test]
fn upgrading_keeps_the_data_and_the_same_passphrase() {
    let (session, data) = Session::open(V1_FIXTURE, V1_FIXTURE_PASSPHRASE).unwrap();
    let upgraded = session.upgraded().unwrap().expect("a version 1 vault can be upgraded");
    assert_eq!(upgraded.format_version(), 2);
    let file = upgraded.seal(&data).unwrap();
    assert_eq!(file_version(&file), Some(VERSION_2));
    let (reopened, reopened_data) = Session::open(&file, V1_FIXTURE_PASSPHRASE).unwrap();
    assert_eq!(reopened_data, sample_data());
    assert_eq!(reopened.format_version(), 2);
    assert!(matches!(Session::open(&file, "wrong"), Err(VaultError::Decrypt)));
}

#[test]
fn upgrading_leaves_the_original_file_openable() {
    let (session, _) = Session::open(V1_FIXTURE, V1_FIXTURE_PASSPHRASE).unwrap();
    session.upgraded().unwrap();
    assert!(Session::open(V1_FIXTURE, V1_FIXTURE_PASSPHRASE).is_ok());
}

#[test]
fn an_upgraded_vault_has_nothing_left_to_upgrade() {
    let upgraded = Session::create("pw", FAST_PARAMS).unwrap();
    assert!(upgraded.upgraded().unwrap().is_none());
}

#[test]
fn the_two_formats_refuse_each_others_files_instead_of_guessing() {
    let (legacy, _) = Session::open(V1_FIXTURE, V1_FIXTURE_PASSPHRASE).unwrap();
    let slotted_file = sealed_vault("pw");
    assert!(matches!(legacy.unseal(&slotted_file), Err(VaultError::NewerFormat)));
    let slotted = Session::create("pw", FAST_PARAMS).unwrap();
    assert!(matches!(slotted.unseal(V1_FIXTURE), Err(VaultError::OlderFormat)));
}

#[test]
fn an_older_app_that_only_knows_version_1_refuses_a_version_2_file() {
    assert!(matches!(Header::decode(&sealed_vault("pw")), Err(VaultError::BadFormat)));
}

#[test]
fn header_round_trips_and_has_a_stable_size() {
    let header = Header { params: FAST_PARAMS, salt: [7; SALT_LEN], nonce: [9; NONCE_LEN] };
    let encoded = header.encode();
    assert_eq!(encoded.len(), 57);
    assert_eq!(Header::decode(&encoded).unwrap(), header);
}

/// Encrypts arbitrary plaintext exactly as the version 1 format did, to simulate older files.
fn seal_raw(keys: &LegacyKeys, plaintext: &[u8]) -> Vec<u8> {
    let header = Header { params: keys.params, salt: keys.salt, nonce: [7; NONCE_LEN] };
    let encoded = header.encode();
    let ciphertext = encrypt(&keys.key, &header.nonce, plaintext, associated_data(&encoded)).unwrap();
    [encoded.as_slice(), &ciphertext].concat()
}

#[test]
fn a_vault_saved_before_accounts_had_ids_still_opens_and_is_given_them() {
    let salt = [3u8; SALT_LEN];
    let keys = LegacyKeys { key: derive_key("pw", &salt, FAST_PARAMS).unwrap(), salt, params: FAST_PARAMS };
    let legacy_accounts = vec![sample_data().accounts[0].account.clone()];
    let old_file = seal_raw(&keys, &serde_json::to_vec(&legacy_accounts).unwrap());
    let (_, data) = Session::open(&old_file, "pw").unwrap();
    assert_eq!(data.accounts.len(), 1);
    assert_eq!(data.accounts[0].account, legacy_accounts[0]);
    assert_eq!(data.accounts[0].id, sample_data().accounts[0].id);
    assert_eq!(data.accounts[0].updated_at_ms, 0);
}
