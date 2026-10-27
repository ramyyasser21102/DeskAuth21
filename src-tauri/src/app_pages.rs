//! Sending the person to a page of the main window (the tray menu's Settings item uses this).

use crate::error::CommandError;
use crate::window::show_main_window;
use tauri::{AppHandle, Emitter};

const MAIN_WINDOW_LABEL: &str = "main";
const OPEN_PAGE_EVENT: &str = "open-page";
const PAGES: [&str; 3] = ["codes", "import", "settings"];

/// Brings the main window forward on `page` (codes, import or settings).
pub fn open_page(app: &AppHandle, page: &str) -> Result<(), CommandError> {
    if !PAGES.contains(&page) {
        return Err(CommandError::InvalidSetting(format!("the page {page}")));
    }
    show_main_window(app);
    app.emit_to(MAIN_WINDOW_LABEL, OPEN_PAGE_EVENT, page).map_err(|error| CommandError::Storage(std::io::Error::other(error.to_string())))
}
