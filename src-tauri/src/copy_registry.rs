//! Remembers the newest code placed on the clipboard so it can be taken back off
//! when the timer runs out, when the vault locks, or when the app quits. Pure.

use zeroize::Zeroizing;

/// A copy that is no longer needed is wiped from memory when dropped.
pub struct CopiedCode {
    pub code: Zeroizing<String>,
}

#[derive(Default)]
pub struct CopyRegistry {
    latest: Option<(u64, CopiedCode)>,
    copies_made: u64,
}

impl CopyRegistry {
    pub const fn new() -> Self {
        Self { latest: None, copies_made: 0 }
    }

    /// Takes over from any earlier copy and returns this copy's number.
    pub fn record(&mut self, code: String) -> u64 {
        self.copies_made += 1;
        self.latest = Some((self.copies_made, CopiedCode { code: Zeroizing::new(code) }));
        self.copies_made
    }

    /// Hands back the remembered code and forgets it. With `only_copy_number`, only if
    /// that copy is still the newest, so an old timer cannot clear a newer copy.
    pub fn take(&mut self, only_copy_number: Option<u64>) -> Option<CopiedCode> {
        let is_wanted = self.latest.as_ref().is_some_and(|(number, _)| only_copy_number.is_none_or(|wanted| wanted == *number));
        if is_wanted { self.latest.take().map(|(_, copy)| copy) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_copy_is_the_one_remembered() {
        let mut registry = CopyRegistry::new();
        let first = registry.record("111111".into());
        let second = registry.record("222222".into());
        assert_ne!(first, second);
        assert_eq!(registry.take(Some(first)).map(|copy| copy.code.to_string()), None);
        assert_eq!(registry.take(Some(second)).map(|copy| copy.code.to_string()), Some("222222".into()));
    }

    #[test]
    fn taking_without_a_number_clears_whatever_is_remembered_once() {
        let mut registry = CopyRegistry::new();
        registry.record("333333".into());
        assert_eq!(registry.take(None).map(|copy| copy.code.to_string()), Some("333333".into()));
        assert!(registry.take(None).is_none());
    }

    #[test]
    fn a_timer_for_a_cleared_copy_finds_nothing() {
        let mut registry = CopyRegistry::new();
        let number = registry.record("444444".into());
        registry.take(None);
        assert!(registry.take(Some(number)).is_none());
    }
}
