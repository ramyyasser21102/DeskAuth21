//! Command-line flags that change how the app starts. Pure.

/// Passed by the autostart entry so a login launch lives in the tray quietly
/// instead of popping the window open.
pub const MINIMIZED_FLAG: &str = "--minimized";

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
    fn a_normal_launch_shows_the_window() {
        assert!(!should_start_hidden(["deskauth21"]));
        assert!(!should_start_hidden(Vec::<String>::new()));
        assert!(!should_start_hidden(["--minimized-ish"]));
    }
}
