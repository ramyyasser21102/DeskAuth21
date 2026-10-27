//! When the vault should lock itself. Pure: elapsed time is passed in.

use std::time::Duration;

pub const ALLOWED_AUTO_LOCK_MINUTES: [u32; 4] = [1, 5, 15, 30];
pub const DEFAULT_AUTO_LOCK_MINUTES: u32 = 5;

const SECONDS_PER_MINUTE: u64 = 60;

/// `None` means "never lock automatically", which is always allowed.
pub fn is_allowed_minutes(minutes: Option<u32>) -> bool {
    minutes.is_none_or(|value| ALLOWED_AUTO_LOCK_MINUTES.contains(&value))
}

pub fn timeout_from_minutes(minutes: Option<u32>) -> Option<Duration> {
    minutes.map(|value| Duration::from_secs(u64::from(value) * SECONDS_PER_MINUTE))
}

pub fn should_auto_lock(idle_for: Duration, timeout: Option<Duration>) -> bool {
    timeout.is_some_and(|limit| idle_for >= limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minutes(count: u64) -> Duration {
        Duration::from_secs(count * SECONDS_PER_MINUTE)
    }

    #[test]
    fn locks_once_idle_time_reaches_the_timeout() {
        let timeout = Some(minutes(5));
        assert!(!should_auto_lock(minutes(4), timeout));
        assert!(should_auto_lock(minutes(5), timeout));
        assert!(should_auto_lock(minutes(60), timeout));
    }

    #[test]
    fn never_locks_when_disabled() {
        assert!(!should_auto_lock(minutes(10_000), None));
    }

    #[test]
    fn only_the_offered_choices_or_never_are_allowed() {
        assert!(is_allowed_minutes(None));
        assert!(is_allowed_minutes(Some(15)));
        assert!(!is_allowed_minutes(Some(0)));
        assert!(!is_allowed_minutes(Some(7)));
    }

    #[test]
    fn converts_minutes_to_a_duration() {
        assert_eq!(timeout_from_minutes(Some(15)), Some(minutes(15)));
        assert_eq!(timeout_from_minutes(None), None);
    }
}
