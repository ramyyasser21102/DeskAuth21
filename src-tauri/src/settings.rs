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
}

impl Default for Settings {
    fn default() -> Self {
        Self { auto_lock_minutes: Some(DEFAULT_AUTO_LOCK_MINUTES) }
    }
}

impl Settings {
    pub fn auto_lock_timeout(&self) -> Option<Duration> {
        timeout_from_minutes(self.auto_lock_minutes)
    }
}

/// A damaged or hand-edited file must never stop the app, or silently disable
/// locking, so anything unreadable or out of range falls back to the defaults.
pub fn parse_settings(json: &str) -> Settings {
    serde_json::from_str::<Settings>(json)
        .ok()
        .filter(|settings| is_allowed_minutes(settings.auto_lock_minutes))
        .unwrap_or_default()
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
            let settings = Settings { auto_lock_minutes: minutes };
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
}
