//! User preferences kept in plain JSON next to the vault. Nothing secret lives here.

use crate::auto_lock_policy::{is_allowed_minutes, timeout_from_minutes, DEFAULT_AUTO_LOCK_MINUTES};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Missing fields take their default, so adding a setting later never breaks an old file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// `None` means never lock automatically.
    pub auto_lock_minutes: Option<u32>,
    /// Folder holding the vault file; `None` means the app's own data directory.
    /// Point it at an iCloud/Dropbox/Syncthing folder to sync between devices.
    pub vault_folder: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { auto_lock_minutes: Some(DEFAULT_AUTO_LOCK_MINUTES), vault_folder: None }
    }
}

impl Settings {
    pub fn auto_lock_timeout(&self) -> Option<Duration> {
        timeout_from_minutes(self.auto_lock_minutes)
    }
}

/// A damaged or hand-edited file must never stop the app, or silently disable
/// locking. An unreadable file gives the defaults; an out-of-range auto-lock
/// falls back to the default value but keeps the other settings, so a bad
/// timeout cannot make the app lose track of where the vault is.
pub fn parse_settings(json: &str) -> Settings {
    let Ok(mut settings) = serde_json::from_str::<Settings>(json) else {
        return Settings::default();
    };
    if !is_allowed_minutes(settings.auto_lock_minutes) {
        settings.auto_lock_minutes = Settings::default().auto_lock_minutes;
    }
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_a_five_minute_lock() {
        assert_eq!(Settings::default().auto_lock_minutes, Some(5));
    }

    #[test]
    fn round_trips_including_never() {
        for minutes in [Some(1), Some(30), None] {
            let settings = Settings { auto_lock_minutes: minutes, vault_folder: None };
            let json = serde_json::to_string(&settings).unwrap();
            assert_eq!(parse_settings(&json), settings);
        }
    }

    #[test]
    fn a_missing_field_means_the_default_but_an_explicit_null_means_never() {
        assert_eq!(parse_settings("{}"), Settings::default());
        assert_eq!(parse_settings(r#"{"auto_lock_minutes":null}"#).auto_lock_minutes, None);
    }

    #[test]
    fn falls_back_to_defaults_for_corrupt_or_out_of_range_files() {
        assert_eq!(parse_settings("not json"), Settings::default());
        assert_eq!(parse_settings(""), Settings::default());
        assert_eq!(parse_settings(r#"{"auto_lock_minutes":0}"#), Settings::default());
        assert_eq!(parse_settings(r#"{"auto_lock_minutes":7}"#), Settings::default());
    }

    #[test]
    fn a_bad_timeout_does_not_make_the_app_forget_the_vault_folder() {
        let parsed = parse_settings(r#"{"auto_lock_minutes":7,"vault_folder":"/sync/DeskAuth"}"#);
        assert_eq!(parsed.auto_lock_minutes, Some(5));
        assert_eq!(parsed.vault_folder.as_deref(), Some("/sync/DeskAuth"));
    }
}
