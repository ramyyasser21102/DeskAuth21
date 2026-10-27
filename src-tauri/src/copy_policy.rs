//! Pure rules for copying a code: which menu ids mean "copy", and when the
//! clipboard may be cleared again. No clipboard or Tauri access here.

use std::time::Duration;

/// Long enough to paste into a login form, short enough that a code does not linger.
pub const DEFAULT_CLIPBOARD_CLEAR_SECONDS: u32 = 15;
pub const ALLOWED_CLIPBOARD_CLEAR_SECONDS: [u32; 4] = [10, 15, 30, 60];

pub fn is_allowed_clear_seconds(seconds: u32) -> bool {
    ALLOWED_CLIPBOARD_CLEAR_SECONDS.contains(&seconds)
}

pub fn clear_delay(seconds: u32) -> Duration {
    Duration::from_secs(u64::from(seconds))
}

const COPY_ITEM_ID_PREFIX: &str = "copy:";

/// Menu items carry the account id, not its position, so a click always copies
/// the account the user saw even if the list was reordered by a sync meanwhile.
pub fn copy_item_id(account_id: &str) -> String {
    format!("{COPY_ITEM_ID_PREFIX}{account_id}")
}

/// `Some(account_id)` when `item_id` is a copy item created by [`copy_item_id`].
pub fn parse_copy_item_id(item_id: &str) -> Option<&str> {
    item_id.strip_prefix(COPY_ITEM_ID_PREFIX).filter(|account_id| !account_id.is_empty())
}

/// Clear only if this is still the most recent copy and the clipboard still
/// holds our code. If a newer copy replaced it, that copy owns the timer; if the
/// user copied something else since, leave their clipboard alone.
pub fn should_clear_clipboard(current_contents: Option<&str>, copied_code: &str, is_latest_copy: bool) -> bool {
    is_latest_copy && current_contents == Some(copied_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_item_ids_round_trip() {
        assert_eq!(parse_copy_item_id(&copy_item_id("abc123")), Some("abc123"));
    }

    #[test]
    fn ignores_other_menu_items_and_empty_ids() {
        assert_eq!(parse_copy_item_id("quit"), None);
        assert_eq!(parse_copy_item_id("autostart"), None);
        assert_eq!(parse_copy_item_id("copy:"), None);
    }

    #[test]
    fn clears_only_when_the_clipboard_still_holds_our_code() {
        assert!(should_clear_clipboard(Some("123456"), "123456", true));
        assert!(!should_clear_clipboard(Some("something else"), "123456", true));
        assert!(!should_clear_clipboard(None, "123456", true));
    }

    #[test]
    fn only_the_offered_clear_times_are_allowed() {
        assert!(is_allowed_clear_seconds(DEFAULT_CLIPBOARD_CLEAR_SECONDS));
        assert!([10, 15, 30, 60].into_iter().all(is_allowed_clear_seconds));
        assert!(!is_allowed_clear_seconds(0));
        assert!(!is_allowed_clear_seconds(45));
        assert_eq!(clear_delay(15), Duration::from_secs(15));
    }

    #[test]
    fn a_newer_copy_takes_over_the_timer() {
        assert!(!should_clear_clipboard(Some("123456"), "123456", false));
    }
}
