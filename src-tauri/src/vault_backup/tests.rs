use super::*;
use crate::test_support::{device, device_with_account, temp_directory, ACME_URI, FAST_KDF, PASSPHRASE};
use crate::vault_actions::{create_new_vault, import_accounts, lock_vault, unlock_vault};

const OTHER_PASSPHRASE: &str = "a completely different passphrase";

fn files_ending_with(directory: &Path, suffix: &str) -> usize {
    std::fs::read_dir(directory).unwrap().filter(|entry| entry.as_ref().unwrap().file_name().to_string_lossy().ends_with(suffix)).count()
}

#[test]
fn a_backup_is_the_encrypted_file_and_works_while_locked() {
    let (state, _) = device_with_account("backup-copy", ACME_URI);
    lock_vault(&state);
    let destination = temp_directory("backup-copy-out").join("copy.backup");
    save_backup(&state, &destination).unwrap();
    assert_eq!(std::fs::read(&destination).unwrap(), std::fs::read(state.vault_path()).unwrap());
}

#[test]
fn restoring_brings_back_the_backups_accounts_and_passphrase_and_keeps_the_old_vault() {
    let (source, _) = device_with_account("restore-source", ACME_URI);
    let backup = temp_directory("restore-backup").join("b.backup");
    save_backup(&source, &backup).unwrap();
    let (target, target_directory) = device("restore-target");
    create_new_vault(&target, OTHER_PASSPHRASE, FAST_KDF).unwrap();
    restore_backup(&target, &backup).unwrap();
    assert!(unlock_vault(&target, OTHER_PASSPHRASE).is_err());
    unlock_vault(&target, PASSPHRASE).unwrap();
    assert_eq!(target.vault_guard().accounts().len(), 1);
    assert_eq!(files_ending_with(&target_directory, BEFORE_RESTORE_SUFFIX), 1);
}

#[test]
fn a_file_that_is_not_a_vault_is_refused_and_nothing_changes() {
    let (state, _) = device_with_account("restore-refused", ACME_URI);
    let before = std::fs::read(state.vault_path()).unwrap();
    let junk = temp_directory("restore-junk").join("notes.backup");
    std::fs::create_dir_all(junk.parent().unwrap()).unwrap();
    std::fs::write(&junk, b"just some notes").unwrap();
    assert!(restore_backup(&state, &junk).is_err());
    assert_eq!(std::fs::read(state.vault_path()).unwrap(), before);
    assert!(state.vault_guard().is_unlocked());
}

#[test]
fn exporting_needs_the_passphrase_again_and_writes_nothing_without_it() {
    let (state, _) = device_with_account("export-wrong", ACME_URI);
    let destination = temp_directory("export-wrong-out").join("accounts.txt");
    assert!(export_accounts(&state, "not the passphrase", &destination).is_err());
    assert!(!destination.exists());
}

#[test]
fn exporting_writes_links_that_import_back() {
    let (state, _) = device_with_account("export-ok", ACME_URI);
    let destination = temp_directory("export-ok-out").join("accounts.txt");
    assert_eq!(export_accounts(&state, PASSPHRASE, &destination).unwrap(), 1);
    let (fresh, _) = device("export-fresh");
    create_new_vault(&fresh, PASSPHRASE, FAST_KDF).unwrap();
    let text = std::fs::read_to_string(&destination).unwrap();
    assert_eq!(import_accounts(&fresh, &text, None).unwrap().added, 1);
}

#[test]
fn exporting_needs_an_unlocked_vault() {
    let (state, _) = device_with_account("export-locked", ACME_URI);
    lock_vault(&state);
    let destination = temp_directory("export-locked-out").join("accounts.txt");
    assert!(export_accounts(&state, PASSPHRASE, &destination).is_err());
    assert!(!destination.exists());
}
