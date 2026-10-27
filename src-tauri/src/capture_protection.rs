//! Asking the system to keep the main window out of screenshots and recordings.
//!
//! This is a request, not a guarantee: macOS and Windows honour it for their own
//! capture tools, other recorders may not, and Linux has no way to do it.

use tauri::WebviewWindow;

pub const fn is_supported() -> bool {
    cfg!(any(target_os = "macos", target_os = "windows"))
}

pub fn apply(window: &WebviewWindow, enabled: bool) {
    if let Err(error) = window.set_content_protected(enabled && is_supported()) {
        eprintln!("could not change screen capture protection: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_supported_only_where_the_system_can_do_it() {
        assert_eq!(is_supported(), cfg!(target_os = "macos") || cfg!(target_os = "windows"));
    }
}
