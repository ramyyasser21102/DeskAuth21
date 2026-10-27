//! When the computer going away from the person should lock the vault. Pure: the
//! platform adapters feed in readings once a second, nothing here reads a clock.

/// Readings arrive every second, so a gap this long on either clock means the whole
/// computer was suspended (a program is never stalled that long otherwise).
pub const SUSPEND_GAP_MILLIS: u128 = 5_000;

/// Notices that the computer slept. Different systems stop different clocks while
/// asleep, so the larger of the two gaps decides. A wall clock set forward by more
/// than the gap also counts, which only ever errs toward locking.
#[derive(Debug, Default)]
pub struct SuspendDetector {
    last_readings: Option<(u128, u128)>,
}

impl SuspendDetector {
    pub fn observe(&mut self, wall_millis: u128, monotonic_millis: u128) -> bool {
        let previous = self.last_readings.replace((wall_millis, monotonic_millis));
        let Some((previous_wall, previous_monotonic)) = previous else {
            return false;
        };
        let gap = wall_millis.saturating_sub(previous_wall).max(monotonic_millis.saturating_sub(previous_monotonic));
        gap >= SUSPEND_GAP_MILLIS
    }
}

/// Reports the moment the screen becomes locked, once, not on every reading after.
#[derive(Debug, Default)]
pub struct ScreenLockDetector {
    was_locked: bool,
}

impl ScreenLockDetector {
    pub fn observe(&mut self, is_locked: bool) -> bool {
        let just_locked = is_locked && !self.was_locked;
        self.was_locked = is_locked;
        just_locked
    }
}

pub fn should_lock(lock_on_sleep_enabled: bool, suspended: bool, screen_just_locked: bool) -> bool {
    lock_on_sleep_enabled && (suspended || screen_just_locked)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: u128 = 1_000;

    #[test]
    fn steady_ticks_are_not_a_suspend() {
        let mut detector = SuspendDetector::default();
        for tick in 0..10 {
            assert!(!detector.observe(tick * SECOND, tick * SECOND));
        }
    }

    #[test]
    fn a_long_wall_clock_gap_is_a_suspend_even_if_the_monotonic_clock_paused() {
        let mut detector = SuspendDetector::default();
        detector.observe(0, 0);
        assert!(detector.observe(3_600 * SECOND, SECOND));
    }

    #[test]
    fn a_long_monotonic_gap_is_a_suspend_even_if_the_wall_clock_paused() {
        let mut detector = SuspendDetector::default();
        detector.observe(0, 0);
        assert!(detector.observe(SECOND, 3_600 * SECOND));
    }

    #[test]
    fn it_reports_once_and_recovers() {
        let mut detector = SuspendDetector::default();
        detector.observe(0, 0);
        assert!(detector.observe(60 * SECOND, 60 * SECOND));
        assert!(!detector.observe(61 * SECOND, 61 * SECOND));
    }

    #[test]
    fn a_clock_going_backwards_is_not_a_suspend() {
        let mut detector = SuspendDetector::default();
        detector.observe(100 * SECOND, 100 * SECOND);
        assert!(!detector.observe(90 * SECOND, 101 * SECOND));
    }

    #[test]
    fn the_first_reading_is_never_a_suspend() {
        assert!(!SuspendDetector::default().observe(1_000_000, 1_000_000));
    }

    #[test]
    fn the_screen_lock_is_reported_once_when_it_starts() {
        let mut detector = ScreenLockDetector::default();
        assert!(!detector.observe(false));
        assert!(detector.observe(true));
        assert!(!detector.observe(true));
        assert!(!detector.observe(false));
        assert!(detector.observe(true));
    }

    #[test]
    fn locking_needs_the_setting_and_a_reason() {
        assert!(should_lock(true, true, false));
        assert!(should_lock(true, false, true));
        assert!(!should_lock(true, false, false));
        assert!(!should_lock(false, true, true));
    }
}
