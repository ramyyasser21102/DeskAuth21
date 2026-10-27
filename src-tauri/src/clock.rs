//! The only place the system clock is read. Everything else takes time as an argument.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch. A clock set before 1970 reads as 0 rather than failing.
pub fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// Milliseconds since the Unix epoch; stamps account changes so devices can order them.
pub fn current_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
}
