//! Shared test helpers for the vault formats.

mod hardware;
mod hardware_slots;
mod layout;
mod legacy;
mod session;
mod slots;

use super::*;
use crate::otp::{Account, Algorithm as OtpAlgorithm};
use crate::record::{AccountRecord, VaultData};
use crate::secret::Secret;

/// Cheap parameters so the tests stay fast; production uses `KdfParams::DEFAULT`.
pub(super) const FAST_PARAMS: KdfParams = KdfParams { memory_kib: 8, passes: 1, lanes: 1 };

/// A real version 1 vault written by the code before slots existed, with `sample_data()` in it.
pub(super) const V1_FIXTURE: &[u8] = include_bytes!("../fixtures/v1_vault.da21");
pub(super) const V1_FIXTURE_PASSPHRASE: &str = "legacy fixture pass";

pub(super) fn sample_data() -> VaultData {
    let account = Account {
        issuer: "Acme".into(),
        name: "a@b".into(),
        secret: Secret::new(b"12345678901234567890".to_vec()),
        algorithm: OtpAlgorithm::Sha1,
        digits: 6,
        period_seconds: 30,
    };
    VaultData { accounts: vec![AccountRecord::new(account, 100)], tombstones: Vec::new() }
}

pub(super) fn sealed_vault(passphrase: &str) -> Vec<u8> {
    Session::create(passphrase, FAST_PARAMS).unwrap().seal(&sample_data()).unwrap()
}
