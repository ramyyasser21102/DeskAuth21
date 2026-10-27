use super::*;

const SCREEN: ScreenRect = ScreenRect { x: 0.0, y: 0.0, width: 2000.0, height: 1200.0 };
const PANEL: (f64, f64) = (360.0, 520.0);

fn icon_at(x: f64, y: f64) -> Option<ScreenRect> {
    Some(ScreenRect { x, y, width: 30.0, height: 24.0 })
}

#[test]
fn opens_centred_under_an_icon_in_the_menu_bar() {
    let (x, y) = panel_position(icon_at(1000.0, 0.0), PANEL, SCREEN);
    assert_eq!(x, 1000.0 + 15.0 - 180.0);
    assert_eq!(y, 24.0 + EDGE_MARGIN);
}

#[test]
fn opens_above_an_icon_on_a_bottom_taskbar() {
    let (_, y) = panel_position(icon_at(1000.0, 1170.0), PANEL, SCREEN);
    assert_eq!(y, 1170.0 - 520.0 - EDGE_MARGIN);
}

#[test]
fn stays_on_screen_when_the_icon_is_near_the_right_edge() {
    let (x, _) = panel_position(icon_at(1990.0, 0.0), PANEL, SCREEN);
    assert_eq!(x, 2000.0 - 360.0 - EDGE_MARGIN);
}

#[test]
fn stays_on_screen_when_the_icon_is_near_the_left_edge() {
    let (x, _) = panel_position(icon_at(0.0, 0.0), PANEL, SCREEN);
    assert_eq!(x, EDGE_MARGIN);
}

#[test]
fn without_an_icon_it_opens_in_the_top_right_corner() {
    assert_eq!(panel_position(None, PANEL, SCREEN), (2000.0 - 360.0 - EDGE_MARGIN, TOP_OFFSET));
}

#[test]
fn works_on_a_second_screen_to_the_left() {
    let left = ScreenRect { x: -1920.0, y: 0.0, width: 1920.0, height: 1080.0 };
    let (x, _) = panel_position(icon_at(-1900.0, 0.0), PANEL, left);
    assert_eq!(x, -1920.0 + EDGE_MARGIN);
}

#[test]
fn a_screen_smaller_than_the_panel_still_gives_a_position() {
    let tiny = ScreenRect { x: 0.0, y: 0.0, width: 300.0, height: 300.0 };
    let (x, y) = panel_position(icon_at(100.0, 0.0), PANEL, tiny);
    assert!(x.is_finite() && y.is_finite());
}

#[test]
fn a_click_opens_a_closed_panel_but_not_one_that_just_closed_itself() {
    assert!(should_open_on_click(false, None));
    assert!(should_open_on_click(false, Some(REOPEN_GUARD_MILLIS + 1)));
    assert!(!should_open_on_click(false, Some(50)));
}

#[test]
fn a_click_never_opens_a_panel_that_is_already_open() {
    assert!(!should_open_on_click(true, None));
}
