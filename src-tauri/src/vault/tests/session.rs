use super::*;
use crate::vault::layout::{file_version, parse, VERSION_2};

const FIRST_SLOT_MEMORY_FIELD: std::ops::Range<usize> = 9..13;
const FIRST_SLOT_SALT_BYTE: usize = 22;

#[test]
fn round_trips_accounts() {
    let bytes = sealed_vault("hunter2");
    let (_, accounts) = Session::open(&bytes, "hunter2").unwrap();
    assert_eq!(accounts, sample_data());
}

#[test]
fn new_vaults_use_the_current_format() {
    let bytes = sealed_vault("pw");
    assert_eq!(file_version(&bytes), Some(VERSION_2));
    assert_eq!(Session::open(&bytes, "pw").unwrap().0.format_version(), 2);
}

#[test]
fn does_not_leak_plaintext() {
    let bytes = sealed_vault("pw");
    assert!(!bytes.windows(4).any(|window| window == b"Acme"));
}

#[test]
fn rejects_wrong_passphrase() {
    let bytes = sealed_vault("right");
    assert!(matches!(Session::open(&bytes, "wrong"), Err(VaultError::Decrypt)));
}

#[test]
fn detects_ciphertext_tampering() {
    let mut bytes = sealed_vault("pw");
    *bytes.last_mut().unwrap() ^= 1;
    assert!(Session::open(&bytes, "pw").is_err());
}

#[test]
fn detects_slot_tampering() {
    let mut bytes = sealed_vault("pw");
    bytes[FIRST_SLOT_SALT_BYTE] ^= 1;
    assert!(Session::open(&bytes, "pw").is_err());
}

#[test]
fn reseal_uses_a_fresh_nonce_and_still_opens() {
    let session = Session::create("pw", FAST_PARAMS).unwrap();
    let first = session.seal(&sample_data()).unwrap();
    let second = session.seal(&sample_data()).unwrap();
    assert_ne!(parse(&first).unwrap().payload_nonce, parse(&second).unwrap().payload_nonce);
    assert!(Session::open(&second, "pw").is_ok());
}

#[test]
fn unseal_reads_a_newer_copy_with_the_same_key_without_the_passphrase() {
    let session = Session::create("pw", FAST_PARAMS).unwrap();
    let newer = session.seal(&sample_data()).unwrap();
    assert_eq!(session.unseal(&newer).unwrap(), sample_data());
}

#[test]
fn unseal_refuses_a_different_vault_even_if_the_passphrase_matches() {
    let ours = Session::create("same passphrase", FAST_PARAMS).unwrap();
    assert!(matches!(ours.unseal(&sealed_vault("same passphrase")), Err(VaultError::Decrypt)));
}

#[test]
fn rejects_garbage_input() {
    assert!(matches!(Session::open(b"nope", "pw"), Err(VaultError::BadFormat)));
}

#[test]
fn rejects_hostile_kdf_costs_before_deriving() {
    let mut bytes = sealed_vault("pw");
    bytes[FIRST_SLOT_MEMORY_FIELD].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(matches!(Session::open(&bytes, "pw"), Err(VaultError::BadParams)));
}
