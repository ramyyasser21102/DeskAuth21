use super::header::{HEADER_LEN, NONCE_LEN};
use super::*;
use crate::otp::Algorithm as OtpAlgorithm;

/// Cheap parameters so the tests stay fast; production uses `KdfParams::DEFAULT`.
const FAST_PARAMS: KdfParams = KdfParams { memory_kib: 8, passes: 1, lanes: 1 };
const MEMORY_FIELD: std::ops::Range<usize> = 5..9;
const LAST_SALT_BYTE: usize = 32;

fn sample_accounts() -> Vec<Account> {
    vec![Account {
        issuer: "Acme".into(),
        name: "a@b".into(),
        secret: b"12345678901234567890".to_vec(),
        algorithm: OtpAlgorithm::Sha1,
        digits: 6,
        period_seconds: 30,
    }]
}

fn sealed_vault(passphrase: &str) -> Vec<u8> {
    Session::create(passphrase, FAST_PARAMS).unwrap().seal(&sample_accounts()).unwrap()
}

#[test]
fn round_trips_accounts() {
    let bytes = sealed_vault("hunter2");
    let (_, accounts) = Session::open(&bytes, "hunter2").unwrap();
    assert_eq!(accounts, sample_accounts());
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
fn detects_header_tampering() {
    let mut bytes = sealed_vault("pw");
    bytes[LAST_SALT_BYTE] ^= 1;
    assert!(Session::open(&bytes, "pw").is_err());
}

#[test]
fn reseal_uses_a_fresh_nonce_and_still_opens() {
    let session = Session::create("pw", FAST_PARAMS).unwrap();
    let first = session.seal(&sample_accounts()).unwrap();
    let second = session.seal(&sample_accounts()).unwrap();
    assert_ne!(first[HEADER_LEN - NONCE_LEN..], second[HEADER_LEN - NONCE_LEN..]);
    assert!(Session::open(&second, "pw").is_ok());
}

#[test]
fn rejects_garbage_input() {
    assert!(matches!(Session::open(b"nope", "pw"), Err(VaultError::BadFormat)));
}

#[test]
fn rejects_hostile_kdf_costs_before_deriving() {
    let mut bytes = sealed_vault("pw");
    bytes[MEMORY_FIELD].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(matches!(Session::open(&bytes, "pw"), Err(VaultError::BadParams)));
}

#[test]
fn header_round_trips_and_has_a_stable_size() {
    let header = Header { params: FAST_PARAMS, salt: [7; SALT_LEN], nonce: [9; NONCE_LEN] };
    let encoded = header.encode();
    assert_eq!(encoded.len(), 57);
    assert_eq!(Header::decode(&encoded).unwrap(), header);
}
