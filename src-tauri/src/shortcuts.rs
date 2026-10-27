//! The global keyboard shortcuts a person can choose. Pure: registering them is `hotkey`'s job.

use serde::{Deserialize, Serialize};
use tauri_plugin_global_shortcut::Shortcut;

/// Cmd on macOS, Ctrl elsewhere. "A" for Authenticator.
pub const DEFAULT_TOGGLE_WINDOW_SHORTCUT: &str = "CmdOrCtrl+Shift+A";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutAction {
    ToggleWindow,
    LockVault,
}

/// `None` means the action has no shortcut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutSettings {
    pub toggle_window: Option<String>,
    pub lock_vault: Option<String>,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self { toggle_window: Some(DEFAULT_TOGGLE_WINDOW_SHORTCUT.to_string()), lock_vault: None }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ShortcutError {
    #[error("\"{0}\" is not a key combination")]
    Unreadable(String),
    #[error("a shortcut needs a modifier key (Ctrl, Alt, Shift or Cmd) so typing never triggers it")]
    NeedsModifier,
    #[error("that combination is already used for another action")]
    AlreadyUsed,
}

impl ShortcutSettings {
    pub fn get(&self, action: ShortcutAction) -> Option<&str> {
        match action {
            ShortcutAction::ToggleWindow => self.toggle_window.as_deref(),
            ShortcutAction::LockVault => self.lock_vault.as_deref(),
        }
    }

    /// The settings with `action` set to `shortcut`; the original is untouched.
    pub fn with(&self, action: ShortcutAction, shortcut: Option<String>) -> Self {
        let mut changed = self.clone();
        match action {
            ShortcutAction::ToggleWindow => changed.toggle_window = shortcut,
            ShortcutAction::LockVault => changed.lock_vault = shortcut,
        }
        changed
    }

    /// Every chosen shortcut, for registering.
    pub fn assigned(&self) -> Vec<(ShortcutAction, &str)> {
        [ShortcutAction::ToggleWindow, ShortcutAction::LockVault]
            .into_iter()
            .filter_map(|action| self.get(action).map(|shortcut| (action, shortcut)))
            .collect()
    }
}

/// Checks a combination typed by the person before it is saved or registered.
pub fn validate_shortcut(action: ShortcutAction, candidate: &str, current: &ShortcutSettings) -> Result<(), ShortcutError> {
    let parsed = parse(candidate)?;
    if parsed.mods.is_empty() {
        return Err(ShortcutError::NeedsModifier);
    }
    let other = match action {
        ShortcutAction::ToggleWindow => ShortcutAction::LockVault,
        ShortcutAction::LockVault => ShortcutAction::ToggleWindow,
    };
    let is_used_by_other = current.get(other).and_then(|text| parse(text).ok()) == Some(parsed);
    if is_used_by_other {
        return Err(ShortcutError::AlreadyUsed);
    }
    Ok(())
}

fn parse(text: &str) -> Result<Shortcut, ShortcutError> {
    text.parse().map_err(|_| ShortcutError::Unreadable(text.to_string()))
}

/// Keeps the file's choices when they can still be used; an unusable one is dropped
/// rather than failing to register at every start.
pub fn sanitize(settings: ShortcutSettings) -> ShortcutSettings {
    let mut kept = ShortcutSettings { toggle_window: None, lock_vault: None };
    for (action, text) in settings.assigned() {
        if validate_shortcut(action, text, &kept).is_ok() {
            kept = kept.with(action, Some(text.to_string()));
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOGGLE: ShortcutAction = ShortcutAction::ToggleWindow;
    const LOCK: ShortcutAction = ShortcutAction::LockVault;

    #[test]
    fn the_window_toggle_has_a_default_and_locking_has_none() {
        let defaults = ShortcutSettings::default();
        assert_eq!(defaults.get(TOGGLE), Some("CmdOrCtrl+Shift+A"));
        assert_eq!(defaults.get(LOCK), None);
    }

    #[test]
    fn accepts_a_modifier_with_a_key() {
        assert_eq!(validate_shortcut(LOCK, "Ctrl+Alt+KeyL", &ShortcutSettings::default()), Ok(()));
    }

    #[test]
    fn rejects_a_bare_key_so_typing_cannot_trigger_it() {
        assert_eq!(validate_shortcut(LOCK, "KeyL", &ShortcutSettings::default()), Err(ShortcutError::NeedsModifier));
    }

    #[test]
    fn rejects_text_that_is_not_a_combination() {
        let result = validate_shortcut(LOCK, "Ctrl+Banana", &ShortcutSettings::default());
        assert_eq!(result, Err(ShortcutError::Unreadable("Ctrl+Banana".to_string())));
    }

    #[test]
    fn rejects_the_combination_another_action_uses_even_when_spelled_differently() {
        let current = ShortcutSettings::default();
        let other_spelling = if cfg!(target_os = "macos") { "shift+super+KeyA" } else { "shift+ctrl+KeyA" };
        assert_eq!(validate_shortcut(LOCK, other_spelling, &current), Err(ShortcutError::AlreadyUsed));
        let spelled_the_same = validate_shortcut(LOCK, "CmdOrCtrl+Shift+A", &current);
        assert_eq!(spelled_the_same, Err(ShortcutError::AlreadyUsed));
    }

    #[test]
    fn an_action_may_keep_its_own_combination() {
        let current = ShortcutSettings::default();
        assert_eq!(validate_shortcut(TOGGLE, "CmdOrCtrl+Shift+A", &current), Ok(()));
    }

    #[test]
    fn with_changes_only_the_chosen_action() {
        let changed = ShortcutSettings::default().with(LOCK, Some("Ctrl+KeyL".to_string()));
        assert_eq!(changed.get(LOCK), Some("Ctrl+KeyL"));
        assert_eq!(changed.get(TOGGLE), Some("CmdOrCtrl+Shift+A"));
    }

    #[test]
    fn a_missing_field_in_an_old_file_takes_its_default() {
        let parsed: ShortcutSettings = serde_json::from_str(r#"{"lock_vault":"Ctrl+KeyL"}"#).unwrap();
        assert_eq!(parsed.toggle_window.as_deref(), Some("CmdOrCtrl+Shift+A"));
    }

    #[test]
    fn sanitizing_drops_unusable_and_duplicate_choices_but_keeps_the_rest() {
        let messy = ShortcutSettings { toggle_window: Some("KeyA".to_string()), lock_vault: Some("Ctrl+KeyL".to_string()) };
        let cleaned = sanitize(messy);
        assert_eq!((cleaned.toggle_window, cleaned.lock_vault.as_deref()), (None, Some("Ctrl+KeyL")));
        let doubled = ShortcutSettings { toggle_window: Some("Ctrl+KeyL".to_string()), lock_vault: Some("Ctrl+KeyL".to_string()) };
        assert_eq!(sanitize(doubled).lock_vault, None);
    }
}
