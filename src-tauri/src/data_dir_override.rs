//! Lets a second copy of the app keep its data somewhere else, so sync can be
//! tested on one computer by running two instances. Pure: the environment is read by the caller.

use std::ffi::OsString;
use std::path::PathBuf;

/// Name of the environment variable that moves the app's data folder.
pub const DATA_DIR_VARIABLE: &str = "DESKAUTH21_DATA_DIR";

/// An empty value counts as unset, because shells export empty variables easily.
pub fn is_overridden(override_value: Option<&OsString>) -> bool {
    override_value.is_some_and(|value| !value.is_empty())
}

pub fn resolve_app_data_dir(default_dir: PathBuf, override_value: Option<OsString>) -> PathBuf {
    match override_value {
        Some(value) if is_overridden(Some(&value)) => PathBuf::from(value),
        _ => default_dir,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_dir() -> PathBuf {
        PathBuf::from("default")
    }

    #[test]
    fn an_empty_value_is_not_an_override() {
        assert!(!is_overridden(None));
        assert!(!is_overridden(Some(&OsString::new())));
        assert!(is_overridden(Some(&OsString::from("/tmp/other"))));
    }

    #[test]
    fn uses_the_default_when_unset() {
        assert_eq!(resolve_app_data_dir(default_dir(), None), default_dir());
    }

    #[test]
    fn uses_the_override_when_set() {
        let overridden = resolve_app_data_dir(default_dir(), Some("second-instance".into()));
        assert_eq!(overridden, PathBuf::from("second-instance"));
    }

    #[test]
    fn ignores_an_empty_override() {
        assert_eq!(resolve_app_data_dir(default_dir(), Some(OsString::new())), default_dir());
    }
}
