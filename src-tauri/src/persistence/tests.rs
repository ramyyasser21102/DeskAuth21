use super::*;

fn temp_dir(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("deskauth21-persistence-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn clean_up(directory: &Path) {
    // Best effort: a leftover temp directory must not fail the test.
    let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn reads_a_small_regular_file() {
    let directory = temp_dir("small");
    let path = directory.join("vault.da21");
    std::fs::write(&path, b"hello").unwrap();
    assert_eq!(read_bounded(&path, 16).unwrap(), b"hello");
    clean_up(&directory);
}

#[test]
fn refuses_a_file_over_the_limit_and_a_directory() {
    let directory = temp_dir("limit");
    let path = directory.join("big.da21");
    std::fs::write(&path, [0u8; 32]).unwrap();
    assert!(read_bounded(&path, 16).is_err());
    assert!(read_bounded(&directory, 16).is_err());
    clean_up(&directory);
}

#[cfg(unix)]
#[test]
fn refuses_a_device_reached_through_a_link() {
    let directory = temp_dir("device");
    let link = directory.join("vault 2.da21");
    std::os::unix::fs::symlink("/dev/zero", &link).unwrap();
    assert!(read_bounded(&link, MAX_VAULT_FILE_BYTES).is_err());
    clean_up(&directory);
}

#[test]
fn writing_replaces_the_file_and_leaves_no_temp_file() {
    let directory = temp_dir("atomic");
    let path = directory.join("vault.da21");
    write_atomically(&path, b"first").unwrap();
    write_atomically(&path, b"second").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"second");
    let names: Vec<_> = std::fs::read_dir(&directory).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(names, vec![std::ffi::OsString::from("vault.da21")]);
    clean_up(&directory);
}

#[cfg(unix)]
#[test]
fn the_written_file_is_private_to_the_owner() {
    use std::os::unix::fs::PermissionsExt;
    let directory = temp_dir("mode");
    let path = directory.join("vault.da21");
    write_atomically(&path, b"secret").unwrap();
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, OWNER_READ_WRITE_ONLY);
    clean_up(&directory);
}

#[test]
fn temp_names_are_random_and_hidden() {
    let path = Path::new("/folder/vault.da21");
    let first = unique_temp_path(path).unwrap();
    let second = unique_temp_path(path).unwrap();
    assert_ne!(first, second);
    assert!(first.file_name().unwrap().to_string_lossy().starts_with(".vault.da21."));
}
