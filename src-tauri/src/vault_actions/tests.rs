use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

const FAST_KDF: KdfParams = KdfParams { memory_kib: 8, passes: 1, lanes: 1 };
const PASSPHRASE: &str = "correct horse";
const ACCOUNT_URI: &str = "otpauth://totp/Acme:a@b?secret=GEZDGNBVGY3TQOJQ&issuer=Acme";

static NEXT_DIRECTORY_ID: AtomicU32 = AtomicU32::new(0);

/// A state whose vault file lives in its own throwaway directory.
fn state_in_temp_directory() -> (AppState, PathBuf) {
    let id = NEXT_DIRECTORY_ID.fetch_add(1, Ordering::SeqCst);
    let directory = std::env::temp_dir().join(format!("deskauth21-test-{}-{id}", std::process::id()));
    (AppState::new(directory.join("vault.da21")), directory)
}

fn unlocked_state() -> (AppState, PathBuf) {
    let (state, directory) = state_in_temp_directory();
    create_new_vault(&state, PASSPHRASE, FAST_KDF).unwrap();
    (state, directory)
}

fn remove(directory: PathBuf) {
    // Best effort: a leftover temp directory must not fail the test.
    let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn created_vault_can_be_locked_and_unlocked() {
    let (state, directory) = unlocked_state();
    lock_vault(&state);
    assert!(matches!(current_codes_now(&state), Err(CommandError::VaultLocked)));
    unlock_vault(&state, PASSPHRASE).unwrap();
    assert!(current_codes_now(&state).unwrap().is_empty());
    remove(directory);
}

#[test]
fn refuses_weak_passphrases_and_existing_vaults() {
    let (state, directory) = unlocked_state();
    assert!(matches!(create_new_vault(&state, PASSPHRASE, FAST_KDF), Err(CommandError::VaultAlreadyExists)));
    let (fresh, fresh_directory) = state_in_temp_directory();
    assert!(matches!(create_new_vault(&fresh, "short", FAST_KDF), Err(CommandError::PassphraseTooShort(_))));
    remove(directory);
    remove(fresh_directory);
}

#[test]
fn wrong_passphrase_keeps_the_vault_locked() {
    let (state, directory) = unlocked_state();
    lock_vault(&state);
    assert!(unlock_vault(&state, "not the passphrase").is_err());
    assert!(!state.vault_guard().is_unlocked());
    remove(directory);
}

#[test]
fn imports_require_an_unlocked_vault() {
    let (state, directory) = unlocked_state();
    lock_vault(&state);
    assert!(matches!(import_accounts(&state, ACCOUNT_URI), Err(CommandError::VaultLocked)));
    remove(directory);
}

#[test]
fn imported_accounts_survive_a_lock_cycle() {
    let (state, directory) = unlocked_state();
    assert_eq!(import_accounts(&state, ACCOUNT_URI).unwrap(), 1);
    lock_vault(&state);
    unlock_vault(&state, PASSPHRASE).unwrap();
    assert_eq!(current_codes_now(&state).unwrap()[0].label, "Acme (a@b)");
    remove(directory);
}

#[test]
fn looks_up_a_single_code_by_position() {
    let (state, directory) = unlocked_state();
    import_accounts(&state, ACCOUNT_URI).unwrap();
    let all_codes = current_codes(&state, 59).unwrap();
    assert_eq!(code_for_account(&state, 0, 59).unwrap(), all_codes[0].code);
    assert!(matches!(code_for_account(&state, 1, 59), Err(CommandError::AccountNotFound(1))));
    lock_vault(&state);
    assert!(matches!(code_for_account(&state, 0, 59), Err(CommandError::VaultLocked)));
    remove(directory);
}

#[test]
fn a_failed_save_rolls_the_import_back() {
    let (_state, directory) = unlocked_state();
    let blocked = AppState::new(directory.join("vault.da21").join("cannot-create-under-a-file"));
    blocked.vault_guard().unlock(Session::create(PASSPHRASE, FAST_KDF).unwrap(), Vec::new());
    assert!(import_accounts(&blocked, ACCOUNT_URI).is_err());
    assert!(blocked.vault_guard().accounts().is_empty());
    remove(directory);
}
