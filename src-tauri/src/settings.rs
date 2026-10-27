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
    /// Ask the system to keep the window out of screenshots and recordings.
    pub capture_protection: bool,
    /// Show dots instead of digits until the person asks for a code to be revealed.
    pub hide_codes: bool,
    /// List codes in the tray menu; off means labels only, and a click still copies.
    pub tray_shows_codes: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_lock_minutes: Some(DEFAULT_AUTO_LOCK_MINUTES),
            vault_folder: None,
            capture_protection: true,
            hide_codes: true,
            tray_shows_codes: false,
        }
    }
}

impl Settings {
    pub fn auto_lock_timeout(&self) -> Option<Duration> {
        timeout_from_minutes(self.auto_lock_minutes)
    }
}

/// What the window gets: the settings plus what this system can do, so it never
/// claims a protection the system cannot give.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SettingsView {
    #[serde(flatten)]
    pub settings: Settings,
    pub capture_protection_supported: bool,
}

impl From<Settings> for SettingsView {
    fn from(settings: Settings) -> Self {
        Self { settings, capture_protection_supported: crate::capture_protection::is_supported() }
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
            let settings = Settings { auto_lock_minutes: minutes, ..Settings::default() };
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
    fn privacy_defaults_protect_the_person_and_old_files_get_them() {
        let defaults = Settings::default();
        assert!(defaults.capture_protection && defaults.hide_codes && !defaults.tray_shows_codes);
        let from_an_old_file = parse_settings(r#"{"auto_lock_minutes":15,"vault_folder":"/sync"}"#);
        assert!(from_an_old_file.capture_protection && from_an_old_file.hide_codes && !from_an_old_file.tray_shows_codes);
    }

    #[test]
    fn privacy_choices_survive_a_round_trip() {
        let chosen = Settings { capture_protection: false, hide_codes: false, tray_shows_codes: true, ..Settings::default() };
        assert_eq!(parse_settings(&serde_json::to_string(&chosen).unwrap()), chosen);
    }

    #[test]
    fn the_window_view_flattens_the_settings_and_reports_support() {
        let json = serde_json::to_value(SettingsView::from(Settings::default())).unwrap();
        assert_eq!(json["hide_codes"], true);
        assert!(json["capture_protection_supported"].is_boolean());
    }

    #[test]
    fn a_bad_timeout_does_not_make_the_app_forget_the_vault_folder() {
        let parsed = parse_settings(r#"{"auto_lock_minutes":7,"vault_folder":"/sync/DeskAuth"}"#);
        assert_eq!(parsed.auto_lock_minutes, Some(5));
        assert_eq!(parsed.vault_folder.as_deref(), Some("/sync/DeskAuth"));
    }
}
