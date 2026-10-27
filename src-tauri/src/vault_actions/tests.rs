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
    (AppState::new(directory.clone()), directory)
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
    assert!(matches!(import_accounts(&state, ACCOUNT_URI, None), Err(CommandError::VaultLocked)));
    remove(directory);
}

#[test]
fn imported_accounts_survive_a_lock_cycle() {
    let (state, directory) = unlocked_state();
    assert_eq!(import_accounts(&state, ACCOUNT_URI, None).unwrap().added, 1);
    lock_vault(&state);
    unlock_vault(&state, PASSPHRASE).unwrap();
    assert_eq!(current_codes_now(&state).unwrap()[0].label, "Acme (a@b)");
    remove(directory);
}

#[test]
fn importing_the_same_export_twice_adds_nothing_new() {
    let (state, directory) = unlocked_state();
    assert_eq!(import_accounts(&state, ACCOUNT_URI, None).unwrap(), ImportSummary { added: 1, skipped_duplicates: 0, skipped_unsupported: 0 });
    assert_eq!(import_accounts(&state, ACCOUNT_URI, None).unwrap(), ImportSummary { added: 0, skipped_duplicates: 1, skipped_unsupported: 0 });
    assert_eq!(current_codes_now(&state).unwrap().len(), 1);
    remove(directory);
}

fn auto_lock_settings(minutes: Option<u32>) -> Settings {
    Settings { auto_lock_minutes: minutes, ..Settings::default() }
}

const MINUTE: std::time::Duration = std::time::Duration::from_secs(60);

#[test]
fn locks_after_the_idle_timeout_but_not_before() {
    let (state, directory) = unlocked_state();
    state.replace_settings(auto_lock_settings(Some(5)));
    let start = std::time::Instant::now();
    state.activity.touch(start);
    assert!(!lock_if_idle(&state, start + 4 * MINUTE));
    assert!(state.vault_guard().is_unlocked());
    assert!(lock_if_idle(&state, start + 5 * MINUTE));
    assert!(!state.vault_guard().is_unlocked());
    remove(directory);
}

#[test]
fn activity_pushes_the_deadline_back() {
    let (state, directory) = unlocked_state();
    state.replace_settings(auto_lock_settings(Some(5)));
    let start = std::time::Instant::now();
    state.activity.touch(start);
    state.activity.touch(start + 4 * MINUTE);
    assert!(!lock_if_idle(&state, start + 8 * MINUTE));
    assert!(lock_if_idle(&state, start + 9 * MINUTE));
    remove(directory);
}

#[test]
fn never_locks_when_auto_lock_is_off_or_the_vault_is_already_locked() {
    let (state, directory) = unlocked_state();
    let start = std::time::Instant::now();
    state.activity.touch(start);
    state.replace_settings(auto_lock_settings(None));
    assert!(!lock_if_idle(&state, start + 600 * MINUTE));
    state.replace_settings(auto_lock_settings(Some(1)));
    lock_vault(&state);
    assert!(!lock_if_idle(&state, start + 600 * MINUTE));
    remove(directory);
}

#[test]
fn updating_auto_lock_saves_applies_and_rejects_bad_values() {
    let (state, directory) = unlocked_state();
    let saved = update_auto_lock(&state, Some(15)).unwrap();
    assert_eq!(saved.auto_lock_minutes, Some(15));
    assert_eq!(state.settings().auto_lock_minutes, Some(15));
    assert_eq!(crate::settings_store::load_settings(&state.settings_path()).auto_lock_minutes, Some(15));
    assert!(matches!(update_auto_lock(&state, Some(7)), Err(CommandError::InvalidSetting(_))));
    assert_eq!(state.settings().auto_lock_minutes, Some(15));
    remove(directory);
}

#[test]
fn looks_up_a_single_code_by_id() {
    let (state, directory) = unlocked_state();
    import_accounts(&state, ACCOUNT_URI, None).unwrap();
    let all_codes = current_codes(&state, 59).unwrap();
    let id = all_codes[0].id.clone();
    assert_eq!(code_for_account(&state, &id, 59).unwrap(), all_codes[0].code);
    let unknown = code_for_account(&state, "nope", 59);
    assert!(matches!(unknown, Err(CommandError::Change(ChangeError::AccountNotFound(_)))));
    lock_vault(&state);
    assert!(matches!(code_for_account(&state, &id, 59), Err(CommandError::VaultLocked)));
    remove(directory);
}

fn first_account_id(state: &AppState) -> String {
    current_codes_now(state).unwrap()[0].id.clone()
}

#[test]
fn renaming_an_account_changes_its_label_and_survives_a_lock_cycle() {
    let (state, directory) = unlocked_state();
    import_accounts(&state, ACCOUNT_URI, None).unwrap();
    let id = first_account_id(&state);
    rename_stored_account(&state, &id, &AccountDetails { issuer: "Work".into(), name: "me@work.com".into(), vendor: Vendor::Other }).unwrap();
    lock_vault(&state);
    unlock_vault(&state, PASSPHRASE).unwrap();
    let view = &current_codes_now(&state).unwrap()[0];
    assert_eq!((view.id.as_str(), view.label.as_str()), (id.as_str(), "Work (me@work.com)"));
    remove(directory);
}

#[test]
fn deleting_an_account_removes_it_for_good_and_it_can_be_imported_again() {
    let (state, directory) = unlocked_state();
    import_accounts(&state, ACCOUNT_URI, None).unwrap();
    let id = first_account_id(&state);
    delete_stored_account(&state, &id).unwrap();
    lock_vault(&state);
    unlock_vault(&state, PASSPHRASE).unwrap();
    assert!(current_codes_now(&state).unwrap().is_empty());
    assert_eq!(import_accounts(&state, ACCOUNT_URI, None).unwrap().added, 1);
    assert_eq!(first_account_id(&state), id);
    remove(directory);
}

#[test]
fn renaming_or_deleting_needs_an_unlocked_vault_and_a_real_account() {
    let (state, directory) = unlocked_state();
    assert!(matches!(delete_stored_account(&state, "nope"), Err(CommandError::Change(_))));
    assert!(matches!(rename_stored_account(&state, "nope", &AccountDetails { issuer: "".into(), name: "x".into(), vendor: Vendor::Other }), Err(CommandError::Change(_))));
    lock_vault(&state);
    assert!(matches!(delete_stored_account(&state, "nope"), Err(CommandError::VaultLocked)));
    remove(directory);
}

#[test]
fn a_failed_save_rolls_the_import_back() {
    let (_state, directory) = unlocked_state();
    // The vault file is a regular file, so nothing can be created "inside" it.
    let blocked = AppState::new(directory.join("vault.da21"));
    blocked.vault_guard().unlock(Session::create(PASSPHRASE, FAST_KDF).unwrap(), VaultData::default());
    assert!(import_accounts(&blocked, ACCOUNT_URI, None).is_err());
    assert!(blocked.vault_guard().accounts().is_empty());
    remove(directory);
}

fn aegis_export(entries: &[(&str, &str, &str)]) -> String {
    let body: Vec<String> = entries
        .iter()
        .map(|(kind, name, secret)| {
            format!(r#"{{"type":"{kind}","name":"{name}","issuer":"Acme","info":{{"secret":"{secret}","algo":"SHA1","digits":6,"period":30}}}}"#)
        })
        .collect();
    format!(r#"{{"version":1,"header":{{"slots":null,"params":null}},"db":{{"version":3,"entries":[{}]}}}}"#, body.join(","))
}

#[test]
fn an_exported_file_tags_every_account_with_its_source_app() {
    let (state, directory) = unlocked_state();
    let export = aegis_export(&[("totp", "a@b", "GEZDGNBVGY3TQOJQ"), ("totp", "c@d", "MFRGGZDFMZTWQ2LK")]);
    assert_eq!(import_accounts(&state, &export, None).unwrap().added, 2);
    assert!(current_codes_now(&state).unwrap().iter().all(|view| view.vendor == Vendor::Aegis));
    remove(directory);
}

#[test]
fn a_plain_link_takes_the_vendor_the_user_chose_and_defaults_to_other() {
    let (state, directory) = unlocked_state();
    import_accounts(&state, ACCOUNT_URI, Some(Vendor::MicrosoftAuthenticator)).unwrap();
    assert_eq!(current_codes_now(&state).unwrap()[0].vendor, Vendor::MicrosoftAuthenticator);
    let other_uri = "otpauth://totp/Globex:c@d?secret=MFRGGZDFMZTWQ2LK&issuer=Globex";
    import_accounts(&state, other_uri, None).unwrap();
    assert_eq!(current_codes_now(&state).unwrap()[1].vendor, Vendor::Other);
    remove(directory);
}

#[test]
fn the_same_account_exported_from_two_apps_is_a_duplicate_not_a_second_entry() {
    let (state, directory) = unlocked_state();
    import_accounts(&state, ACCOUNT_URI, Some(Vendor::GoogleAuthenticator)).unwrap();
    let again = import_accounts(&state, &aegis_export(&[("totp", "a@b", "GEZDGNBVGY3TQOJQ")]), None).unwrap();
    assert_eq!((again.added, again.skipped_duplicates), (0, 1));
    assert_eq!(current_codes_now(&state).unwrap()[0].vendor, Vendor::GoogleAuthenticator);
    remove(directory);
}

#[test]
fn unsupported_entries_are_reported_without_failing_the_import() {
    let (state, directory) = unlocked_state();
    let export = aegis_export(&[("totp", "a@b", "GEZDGNBVGY3TQOJQ"), ("hotp", "c@d", "MFRGGZDFMZTWQ2LK"), ("steam", "e@f", "ONSWG4TFOQ")]);
    let summary = import_accounts(&state, &export, None).unwrap();
    assert_eq!((summary.added, summary.skipped_unsupported), (1, 2));
    remove(directory);
}

#[test]
fn an_unrecognised_file_fails_with_a_helpful_message_and_changes_nothing() {
    let (state, directory) = unlocked_state();
    let result = import_accounts(&state, "just some words", None);
    assert!(matches!(result, Err(CommandError::Import(_))));
    assert!(current_codes_now(&state).unwrap().is_empty());
    remove(directory);
}
