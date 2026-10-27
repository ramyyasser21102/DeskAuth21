//! Command-line flags that change how the app starts. Pure.

/// Passed by the autostart entry so a login launch lives in the tray quietly
/// instead of popping the window open.
pub const MINIMIZED_FLAG: &str = "--minimized";

/// Opens the quick panel as soon as the app starts; handy for trying the panel and for scripts.
pub const SHOW_PANEL_FLAG: &str = "--show-panel";

pub fn should_show_panel<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|arg| arg.as_ref() == SHOW_PANEL_FLAG)
}

pub fn should_start_hidden<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|arg| arg.as_ref() == MINIMIZED_FLAG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_the_minimized_flag_anywhere_in_the_arguments() {
        assert!(should_start_hidden(["deskauth21", "--minimized"]));
        assert!(should_start_hidden(["--minimized", "--other"]));
    }

    #[test]
    fn detects_the_show_panel_flag() {
        assert!(should_show_panel(["deskauth21", "--show-panel"]));
        assert!(!should_show_panel(["deskauth21", "--minimized"]));
    }

    #[test]
    fn a_normal_launch_shows_the_window() {
        assert!(!should_start_hidden(["deskauth21"]));
        assert!(!should_start_hidden(Vec::<String>::new()));
        assert!(!should_start_hidden(["--minimized-ish"]));
    }
}
