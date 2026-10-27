//! Decides what the show/hide hotkey should do. Pure.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityAction {
    Show,
    Hide,
}

/// Hide only when the window is both visible and in front. A visible window that
/// is buried behind others should come forward instead of vanishing, which is
/// what someone pressing the hotkey to find their code wants.
pub fn action_for_toggle(is_visible: bool, is_focused: bool) -> VisibilityAction {
    if is_visible && is_focused {
        VisibilityAction::Hide
    } else {
        VisibilityAction::Show
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_only_a_visible_focused_window() {
        assert_eq!(action_for_toggle(true, true), VisibilityAction::Hide);
    }

    #[test]
    fn shows_a_hidden_window() {
        assert_eq!(action_for_toggle(false, false), VisibilityAction::Show);
    }

    #[test]
    fn brings_a_buried_window_forward() {
        assert_eq!(action_for_toggle(true, false), VisibilityAction::Show);
    }
}
