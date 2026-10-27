//! The quick panel: a small window under the menu bar icon with a search box and every
//! account. It opens on a click on the icon, closes when it loses focus, and is the same
//! web page as the main window shown in a different mode.

use crate::clock::current_unix_millis;
use crate::panel_position::{panel_position, should_open_on_click, ScreenRect};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window};

pub const PANEL_LABEL: &str = "panel";
const PANEL_WIDTH: f64 = 360.0;
const PANEL_HEIGHT: f64 = 520.0;
const NEVER: u64 = 0;

/// Where the icon last was, so the keyboard shortcut can open the panel in the same place.
static LAST_ANCHOR: Mutex<Option<ScreenRect>> = Mutex::new(None);
static CLOSED_BY_FOCUS_LOSS_AT: AtomicU64 = AtomicU64::new(NEVER);

fn panel(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(PANEL_LABEL)
}

/// Made once, hidden, so it opens instantly. It is protected from screen capture like the main window.
pub fn create(app: &AppHandle, is_capture_protected: bool) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, PANEL_LABEL, WebviewUrl::App("index.html?view=panel".into()))
        .title("DeskAuth21 quick panel")
        .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .shadow(true)
        .content_protected(is_capture_protected)
        .build()?;
    Ok(())
}

/// Window-event hook: the panel goes away when the person clicks anywhere else.
pub fn hide_when_focus_is_lost(window: &Window) {
    if window.label() == PANEL_LABEL && window.hide().is_ok() {
        CLOSED_BY_FOCUS_LOSS_AT.store(current_unix_millis(), Ordering::SeqCst);
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(Err(error)) = panel(app).map(|window| window.hide()) {
        eprintln!("could not hide the quick panel: {error}");
    }
}

/// Opens the panel, or closes it if it is open. `anchor` is where to point at; without one the
/// last icon position is used, and failing that the top right corner.
pub fn toggle(app: &AppHandle, anchor: Option<ScreenRect>) {
    let Some(window) = panel(app) else { return };
    if window.is_visible().unwrap_or(false) {
        hide(app);
    } else {
        show(app, &window, anchor);
    }
}

fn show(app: &AppHandle, window: &WebviewWindow, anchor: Option<ScreenRect>) {
    let anchor = anchor.or_else(|| *LAST_ANCHOR.lock().unwrap_or_else(PoisonError::into_inner));
    place(app, window, anchor);
    if let Err(error) = window.show().and_then(|()| window.set_focus()) {
        eprintln!("could not show the quick panel: {error}");
    }
}

fn place(app: &AppHandle, window: &WebviewWindow, anchor: Option<ScreenRect>) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let centre = anchor.map(|icon| (icon.x + icon.width / 2.0, icon.y + icon.height / 2.0));
    let monitor = centre.and_then(|(x, y)| app.monitor_from_point(x, y).ok().flatten()).or_else(|| app.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let screen = ScreenRect { x: f64::from(monitor.position().x), y: f64::from(monitor.position().y), width: f64::from(monitor.size().width), height: f64::from(monitor.size().height) };
    let (x, y) = panel_position(anchor, (PANEL_WIDTH * scale, PANEL_HEIGHT * scale), screen);
    if let Err(error) = window.set_position(PhysicalPosition::new(x, y)) {
        eprintln!("could not place the quick panel: {error}");
    }
}

/// Tray-icon hook: a left click on the icon opens (or closes) the panel under it.
pub fn handle_tray_event(tray: &TrayIcon, event: TrayIconEvent) {
    let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, rect, .. } = event else { return };
    let app = tray.app_handle();
    let scale = app.primary_monitor().ok().flatten().map_or(1.0, |monitor| monitor.scale_factor());
    let (position, size) = (rect.position.to_physical::<f64>(scale), rect.size.to_physical::<f64>(scale));
    let anchor = ScreenRect { x: position.x, y: position.y, width: size.width, height: size.height };
    *LAST_ANCHOR.lock().unwrap_or_else(PoisonError::into_inner) = Some(anchor);
    let is_visible = panel(app).is_some_and(|window| window.is_visible().unwrap_or(false));
    if should_open_on_click(is_visible, millis_since_focus_loss()) {
        toggle(app, Some(anchor));
    }
}

fn millis_since_focus_loss() -> Option<u64> {
    match CLOSED_BY_FOCUS_LOSS_AT.load(Ordering::SeqCst) {
        NEVER => None,
        closed_at => Some(current_unix_millis().saturating_sub(closed_at)),
    }
}
