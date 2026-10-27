//! Pure rules for copying a code: which menu ids mean "copy", and when the
//! clipboard may be cleared again. No clipboard or Tauri access here.

use std::time::Duration;

/// Long enough to paste into a login form, short enough that a code does not linger.
pub const CLIPBOARD_CLEAR_DELAY: Duration = Duration::from_secs(30);

const COPY_ITEM_ID_PREFIX: &str = "copy:";

pub fn copy_item_id(account_index: usize) -> String {
    format!("{COPY_ITEM_ID_PREFIX}{account_index}")
}

/// `Some(index)` when `item_id` is a copy item created by [`copy_item_id`].
pub fn parse_copy_item_id(item_id: &str) -> Option<usize> {
    item_id.strip_prefix(COPY_ITEM_ID_PREFIX)?.parse().ok()
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
        assert_eq!(parse_copy_item_id(&copy_item_id(0)), Some(0));
        assert_eq!(parse_copy_item_id(&copy_item_id(12)), Some(12));
    }

    #[test]
    fn ignores_other_menu_items_and_malformed_ids() {
        assert_eq!(parse_copy_item_id("quit"), None);
        assert_eq!(parse_copy_item_id("copy:"), None);
        assert_eq!(parse_copy_item_id("copy:abc"), None);
        assert_eq!(parse_copy_item_id("copy:-1"), None);
    }

    #[test]
    fn clears_only_when_the_clipboard_still_holds_our_code() {
        assert!(should_clear_clipboard(Some("123456"), "123456", true));
        assert!(!should_clear_clipboard(Some("something else"), "123456", true));
        assert!(!should_clear_clipboard(None, "123456", true));
    }

    #[test]
    fn a_newer_copy_takes_over_the_timer() {
        assert!(!should_clear_clipboard(Some("123456"), "123456", false));
    }
}
