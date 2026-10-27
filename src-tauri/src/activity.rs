//! Tracks when the user last did something, on a monotonic clock so changing
//! the system time cannot hold the vault open or lock it early.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

pub struct ActivityClock {
    last_activity: Mutex<Instant>,
}

impl ActivityClock {
    pub fn new(now: Instant) -> Self {
        Self { last_activity: Mutex::new(now) }
    }

    pub fn touch(&self, now: Instant) {
        *self.last_activity.lock().unwrap_or_else(PoisonError::into_inner) = now;
    }

    pub fn idle_for(&self, now: Instant) -> Duration {
        let last = *self.last_activity.lock().unwrap_or_else(PoisonError::into_inner);
        now.saturating_duration_since(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECOND: Duration = Duration::from_secs(1);

    #[test]
    fn measures_time_since_the_last_touch() {
        let start = Instant::now();
        let clock = ActivityClock::new(start);
        assert_eq!(clock.idle_for(start + 30 * SECOND), 30 * SECOND);
        clock.touch(start + 30 * SECOND);
        assert_eq!(clock.idle_for(start + 40 * SECOND), 10 * SECOND);
    }

    #[test]
    fn a_clock_reading_before_the_last_touch_counts_as_zero_idle() {
        let start = Instant::now();
        let clock = ActivityClock::new(start + 10 * SECOND);
        assert_eq!(clock.idle_for(start), Duration::ZERO);
    }
}
