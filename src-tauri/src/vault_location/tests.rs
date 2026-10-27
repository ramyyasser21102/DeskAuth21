use super::*;
use crate::settings_store::load_settings;
use crate::vault::KdfParams;
use crate::vault_actions::{create_new_vault, current_codes_now, import_accounts, lock_vault, unlock_vault};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

const FAST_KDF: KdfParams = KdfParams { memory_kib: 8, passes: 1, lanes: 1 };
const PASSPHRASE: &str = "correct horse";
const ACCOUNT_URI: &str = "otpauth://totp/Acme:a@b?secret=GEZDGNBVGY3TQOJQ&issuer=Acme";

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

fn temp_directory(label: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("deskauth21-location-{}-{id}-{label}", std::process::id()))
}

/// One simulated device: its own app data directory and its own in-memory state.
fn device(label: &str) -> (AppState, PathBuf) {
    let directory = temp_directory(label);
    (AppState::new(directory.clone()), directory)
}

fn device_with_one_account(label: &str) -> (AppState, PathBuf) {
    let (state, directory) = device(label);
    create_new_vault(&state, PASSPHRASE, FAST_KDF).unwrap();
    import_accounts(&state, ACCOUNT_URI).unwrap();
    (state, directory)
}

fn remove_all(directories: &[PathBuf]) {
    for directory in directories {
        // Best effort: a leftover temp directory must not fail the test.
        let _ = std::fs::remove_dir_all(directory);
    }
}

#[test]
fn moving_into_an_empty_folder_keeps_the_vault_unlocked_and_removes_the_old_copy() {
    let (state, data_dir) = device_with_one_account("move-device");
    let old_path = state.vault_path();
    let shared = temp_directory("move-shared");
    assert_eq!(change_vault_folder(&state, &shared).unwrap(), RelocationOutcome::Moved);
    assert!(shared.join("vault.da21").exists());
    assert!(!old_path.exists());
    assert_eq!(current_codes_now(&state).unwrap().len(), 1);
    lock_vault(&state);
    unlock_vault(&state, PASSPHRASE).unwrap();
    assert_eq!(current_codes_now(&state).unwrap().len(), 1);
    remove_all(&[data_dir, shared]);
}

#[test]
fn the_chosen_folder_is_saved_for_the_next_launch() {
    let (state, data_dir) = device_with_one_account("persist-device");
    let shared = temp_directory("persist-shared");
    change_vault_folder(&state, &shared).unwrap();
    let saved = load_settings(&state.settings_path());
    assert_eq!(saved.vault_folder.as_deref(), shared.to_str());
    remove_all(&[data_dir, shared]);
}

#[test]
fn a_second_device_adopts_the_existing_vault_and_unlocks_it() {
    let (first, first_dir) = device_with_one_account("first-device");
    let shared = temp_directory("adopt-shared");
    change_vault_folder(&first, &shared).unwrap();
    let (second, second_dir) = device("second-device");
    assert_eq!(change_vault_folder(&second, &shared).unwrap(), RelocationOutcome::UsedExisting);
    unlock_vault(&second, PASSPHRASE).unwrap();
    assert_eq!(current_codes_now(&second).unwrap()[0].label, "Acme (a@b)");
    remove_all(&[first_dir, second_dir, shared]);
}

#[test]
fn adopting_a_vault_never_overwrites_the_one_already_on_this_device() {
    let (first, first_dir) = device_with_one_account("keep-first");
    let shared = temp_directory("keep-shared");
    change_vault_folder(&first, &shared).unwrap();
    let (second, second_dir) = device_with_one_account("keep-second");
    let second_own_vault = second.vault_path();
    change_vault_folder(&second, &shared).unwrap();
    assert!(second_own_vault.exists());
    remove_all(&[first_dir, second_dir, shared]);
}

#[test]
fn choosing_a_folder_first_then_creating_puts_the_new_vault_there() {
    let (state, data_dir) = device("select-device");
    let shared = temp_directory("select-shared");
    assert_eq!(change_vault_folder(&state, &shared).unwrap(), RelocationOutcome::Selected);
    create_new_vault(&state, PASSPHRASE, FAST_KDF).unwrap();
    assert!(shared.join("vault.da21").exists());
    assert!(!data_dir.join("vault.da21").exists());
    remove_all(&[data_dir, shared]);
}

#[test]
fn choosing_the_current_folder_changes_nothing() {
    let (state, data_dir) = device_with_one_account("same-device");
    assert_eq!(change_vault_folder(&state, &data_dir).unwrap(), RelocationOutcome::Unchanged);
    assert_eq!(current_codes_now(&state).unwrap().len(), 1);
    remove_all(&[data_dir]);
}
