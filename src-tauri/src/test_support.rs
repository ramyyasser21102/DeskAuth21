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
