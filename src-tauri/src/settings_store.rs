//! Reading and writing the settings file.

use crate::persistence::write_atomically;
use crate::settings::{parse_settings, Settings};
use std::path::Path;

/// A missing file is normal on first run; an unreadable one falls back to defaults.
pub fn load_settings(path: &Path) -> Settings {
    match std::fs::read_to_string(path) {
        Ok(json) => parse_settings(&json),
        Err(_) => Settings::default(),
    }
}

pub fn save_settings(path: &Path, settings: &Settings) -> std::io::Result<()> {
    let json = serde_json::to_vec_pretty(settings).map_err(std::io::Error::other)?;
    write_atomically(path, &json)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("deskauth21-settings-{}-{name}", std::process::id())).join("settings.json")
    }

    #[test]
    fn saved_settings_load_back() {
        let path = temp_path("roundtrip");
        let settings = Settings { auto_lock_minutes: Some(15), vault_folder: Some("/sync".into()), ..Settings::default() };
        save_settings(&path, &settings).unwrap();
        assert_eq!(load_settings(&path), settings);
        // Best effort: a leftover temp directory must not fail the test.
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn a_missing_file_gives_the_defaults() {
        assert_eq!(load_settings(&temp_path("missing")), Settings::default());
    }
}
