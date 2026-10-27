//! Where the quick panel appears, and when a click on the tray icon should open it. Pure:
//! screen sizes and times are passed in.

/// A rectangle on screen, in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Space kept between the panel and the screen edge or the icon.
const EDGE_MARGIN: f64 = 8.0;
/// Without an icon to point at (the keyboard shortcut), start this far below the top of the screen,
/// which clears the menu bar.
const TOP_OFFSET: f64 = 40.0;
/// A click on the icon while the panel is open first takes focus away from the panel, which
/// closes it, and then arrives as a click. Within this time of a close that click was meant to close.
const REOPEN_GUARD_MILLIS: u64 = 400;

fn clamp(value: f64, low: f64, high: f64) -> f64 {
    value.max(low).min(high.max(low))
}

/// Top-left corner for a panel of `panel` size. Under a tray icon in the top half of the screen
/// (macOS menu bar), above it in the bottom half (Windows taskbar), centred on the icon, and kept
/// on screen. With no icon, in the top right corner.
pub fn panel_position(anchor: Option<ScreenRect>, panel: (f64, f64), screen: ScreenRect) -> (f64, f64) {
    let (width, height) = panel;
    let left_limit = screen.x + EDGE_MARGIN;
    let right_limit = screen.x + screen.width - width - EDGE_MARGIN;
    let Some(icon) = anchor else {
        return (clamp(right_limit, left_limit, right_limit), screen.y + TOP_OFFSET);
    };
    let x = clamp(icon.x + icon.width / 2.0 - width / 2.0, left_limit, right_limit);
    let icon_is_in_top_half = icon.y + icon.height / 2.0 < screen.y + screen.height / 2.0;
    let y = if icon_is_in_top_half { icon.y + icon.height + EDGE_MARGIN } else { icon.y - height - EDGE_MARGIN };
    let bottom_limit = screen.y + screen.height - height - EDGE_MARGIN;
    (x, clamp(y, screen.y + EDGE_MARGIN, bottom_limit))
}

/// Whether a click on the icon should open the panel. If the panel just closed because the
/// click took focus away from it, this click is the one that closed it and must not reopen it.
pub fn should_open_on_click(is_visible: bool, millis_since_closed: Option<u64>) -> bool {
    !is_visible && millis_since_closed.is_none_or(|elapsed| elapsed > REOPEN_GUARD_MILLIS)
}

#[cfg(test)]
mod tests;
