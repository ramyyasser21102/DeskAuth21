//! Shared helpers for tests that need real files and simulated devices.

use crate::state::AppState;
use crate::vault::KdfParams;
use crate::vault_actions::{create_new_vault, import_accounts};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

/// Cheap Argon2 settings so tests stay fast; production uses `KdfParams::DEFAULT`.
pub const FAST_KDF: KdfParams = KdfParams { memory_kib: 8, passes: 1, lanes: 1 };
pub const PASSPHRASE: &str = "correct horse";
pub const ACME_URI: &str = "otpauth://totp/Acme:a@b?secret=GEZDGNBVGY3TQOJQ&issuer=Acme";
pub const GLOBEX_URI: &str = "otpauth://totp/Globex:c@d?secret=MFRGGZDFMZTWQ2LK&issuer=Globex";

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

pub fn temp_directory(label: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("deskauth21-test-{}-{id}-{label}", std::process::id()))
}

/// One simulated device: its own app data directory and its own in-memory state.
pub fn device(label: &str) -> (AppState, PathBuf) {
    let directory = temp_directory(label);
    (AppState::new(directory.clone()), directory)
}

pub fn device_with_account(label: &str, uri: &str) -> (AppState, PathBuf) {
    let (state, directory) = device(label);
    create_new_vault(&state, PASSPHRASE, FAST_KDF).unwrap();
    import_accounts(&state, uri, None).unwrap();
    (state, directory)
}

pub fn remove_all(directories: &[PathBuf]) {
    for directory in directories {
        // Best effort: a leftover temp directory must not fail the test.
        let _ = std::fs::remove_dir_all(directory);
    }
}

/// A stand-in for a security key: it derives secrets from its own master secret, can be
/// unplugged, counts how often it was used, and refuses credentials another key made.
pub struct FakeKey {
    id: u8,
    master_secret: [u8; 32],
    pub is_plugged_in: std::cell::Cell<bool>,
    pub registrations: std::cell::Cell<u32>,
    pub derivations: std::cell::Cell<u32>,
    required_pin: Option<String>,
}

impl FakeKey {
    pub fn new(id: u8) -> Self {
        Self { id, master_secret: [id; 32], is_plugged_in: std::cell::Cell::new(true), registrations: std::cell::Cell::new(0), derivations: std::cell::Cell::new(0), required_pin: None }
    }

    pub fn with_pin(id: u8, pin: &str) -> Self {
        Self { required_pin: Some(pin.to_string()), ..Self::new(id) }
    }

    fn check(&self, pin: Option<&str>) -> Result<(), crate::key_device::KeyError> {
        use crate::key_device::KeyError;
        if !self.is_plugged_in.get() {
            return Err(KeyError::NoKey);
        }
        match (&self.required_pin, pin) {
            (Some(required), Some(given)) if required == given => Ok(()),
            (Some(_), _) => Err(KeyError::PinProblem),
            (None, _) => Ok(()),
        }
    }
}

impl crate::key_device::KeyDevice for FakeKey {
    fn register(&self, pin: Option<&str>) -> Result<Vec<u8>, crate::key_device::KeyError> {
        self.check(pin)?;
        self.registrations.set(self.registrations.get() + 1);
        Ok([&[self.id][..], &self.registrations.get().to_be_bytes()].concat())
    }

    fn derive_secret(&self, credential_id: &[u8], salt: &[u8; 32], pin: Option<&str>) -> Result<[u8; 32], crate::key_device::KeyError> {
        use sha2::{Digest, Sha256};
        self.check(pin)?;
        if credential_id.first() != Some(&self.id) {
            return Err(crate::key_device::KeyError::NotThisKey);
        }
        self.derivations.set(self.derivations.get() + 1);
        Ok(Sha256::new().chain_update(self.master_secret).chain_update(credential_id).chain_update(salt).finalize().into())
    }
}
