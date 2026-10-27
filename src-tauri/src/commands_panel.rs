//! Commands for the quick panel and for sending the person to a page of the main window.

use crate::error::CommandError;
use crate::quick_panel;
use crate::window::show_main_window;
use tauri::{AppHandle, Emitter};

const MAIN_WINDOW_LABEL: &str = "main";
const OPEN_PAGE_EVENT: &str = "open-page";
const PAGES: [&str; 3] = ["codes", "import", "settings"];

#[tauri::command]
pub fn hide_quick_panel(app: AppHandle) {
    quick_panel::hide(&app);
}

/// Brings the main window forward on `page` (codes, import or settings) and closes the panel.
#[tauri::command]
pub fn open_app_page(page: String, app: AppHandle) -> Result<(), CommandError> {
    if !PAGES.contains(&page.as_str()) {
        return Err(CommandError::InvalidSetting(format!("the page {page}")));
    }
    quick_panel::hide(&app);
    show_main_window(&app);
    app.emit_to(MAIN_WINDOW_LABEL, OPEN_PAGE_EVENT, page).map_err(|error| CommandError::Storage(std::io::Error::other(error.to_string())))
}
