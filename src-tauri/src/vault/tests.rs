use super::header::{associated_data, Header, HEADER_LEN, NONCE_LEN};
use super::*;
use crate::otp::{Account, Algorithm as OtpAlgorithm};
use crate::record::AccountRecord;

/// Cheap parameters so the tests stay fast; production uses `KdfParams::DEFAULT`.
const FAST_PARAMS: KdfParams = KdfParams { memory_kib: 8, passes: 1, lanes: 1 };
const MEMORY_FIELD: std::ops::Range<usize> = 5..9;
const LAST_SALT_BYTE: usize = 32;

fn sample_data() -> VaultData {
    let account = Account {
        issuer: "Acme".into(),
        name: "a@b".into(),
        secret: b"12345678901234567890".to_vec(),
        algorithm: OtpAlgorithm::Sha1,
        digits: 6,
        period_seconds: 30,
    };
    VaultData { accounts: vec![AccountRecord::new(account, 100)], tombstones: Vec::new() }
}

fn sealed_vault(passphrase: &str) -> Vec<u8> {
    Session::create(passphrase, FAST_PARAMS).unwrap().seal(&sample_data()).unwrap()
}

#[test]
fn round_trips_accounts() {
    let bytes = sealed_vault("hunter2");
    let (_, accounts) = Session::open(&bytes, "hunter2").unwrap();
    assert_eq!(accounts, sample_data());
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
    let first = session.seal(&sample_data()).unwrap();
    let second = session.seal(&sample_data()).unwrap();
    assert_ne!(first[HEADER_LEN - NONCE_LEN..], second[HEADER_LEN - NONCE_LEN..]);
    assert!(Session::open(&second, "pw").is_ok());
}

#[test]
fn unseal_reads_a_newer_copy_with_the_same_key_without_the_passphrase() {
    let session = Session::create("pw", FAST_PARAMS).unwrap();
    let newer = session.seal(&sample_data()).unwrap();
    assert_eq!(session.unseal(&newer).unwrap(), sample_data());
}

#[test]
fn unseal_refuses_a_vault_made_with_a_different_salt_even_if_the_passphrase_matches() {
    let ours = Session::create("same passphrase", FAST_PARAMS).unwrap();
    let theirs = sealed_vault("same passphrase");
    assert!(matches!(ours.unseal(&theirs), Err(VaultError::Decrypt)));
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

/// Encrypts arbitrary plaintext exactly as `seal` would, to simulate older vault files.
fn seal_raw(session: &Session, plaintext: &[u8]) -> Vec<u8> {
    let header = Header { params: session.params, salt: session.salt, nonce: [7; NONCE_LEN] };
    let encoded = header.encode();
    let payload = Payload { msg: plaintext, aad: associated_data(&encoded) };
    let ciphertext = build_cipher(&session.key).unwrap().encrypt((&header.nonce).into(), payload).unwrap();
    [encoded.as_slice(), &ciphertext].concat()
}

#[test]
fn a_vault_saved_before_accounts_had_ids_still_opens_and_is_upgraded() {
    let session = Session::create("pw", FAST_PARAMS).unwrap();
    let legacy_accounts = vec![sample_data().accounts[0].account.clone()];
    let old_file = seal_raw(&session, &serde_json::to_vec(&legacy_accounts).unwrap());
    let (_, data) = Session::open(&old_file, "pw").unwrap();
    assert_eq!(data.accounts.len(), 1);
    assert_eq!(data.accounts[0].account, legacy_accounts[0]);
    assert_eq!(data.accounts[0].id, sample_data().accounts[0].id);
    assert_eq!(data.accounts[0].updated_at_ms, 0);
}
