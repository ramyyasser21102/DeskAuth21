use super::*;

fn view(id: &str, label: &str, code: &str, seconds_left: u64) -> CodeView {
    CodeView {
        id: id.into(),
        label: label.into(),
        issuer: String::new(),
        name: label.into(),
        vendor: crate::vendor::Vendor::Other,
        code: code.into(),
        seconds_left,
    }
}

fn many_views(count: usize) -> Vec<CodeView> {
    (0..count).map(|index| view(&format!("id{index}"), &format!("Service {index}"), "123456", 20)).collect()
}

#[test]
fn truncates_by_characters_not_bytes_and_marks_the_cut() {
    assert_eq!(truncate_label("short", 26), "short");
    assert_eq!(truncate_label("GitHub (alice.smith@example-company.com)", 20), "GitHub (alice.smith…");
    assert_eq!(truncate_label("ééééé", 3), "éé…");
}

#[test]
fn a_row_shows_the_label_and_the_code_in_two_halves_without_a_per_row_countdown() {
    let model = build_tray_model(VaultStatus::Unlocked, &[view("a", "Acme (a@b)", "123456", 9)], false);
    assert_eq!(model.rows[0].text, "Acme (a@b)   123 456");
    assert!(!model.rows[0].text.contains("(9s)"));
}

#[test]
fn one_countdown_shows_the_soonest_expiry() {
    let views = [view("a", "A", "111111", 20), view("b", "B", "222222", 7)];
    let model = build_tray_model(VaultStatus::Unlocked, &views, false);
    assert_eq!(model.countdown_text.as_deref(), Some("Codes refresh in 7s"));
}

#[test]
fn a_long_list_is_cut_to_the_limit_and_the_rest_are_counted() {
    let model = build_tray_model(VaultStatus::Unlocked, &many_views(20), false);
    assert_eq!(model.rows.len(), MAX_TRAY_ACCOUNTS);
    assert_eq!(model.hidden_accounts, 20 - MAX_TRAY_ACCOUNTS);
    assert_eq!(more_accounts_text(12), "…and 12 more — open DeskAuth21");
}

#[test]
fn a_short_list_hides_nothing() {
    let model = build_tray_model(VaultStatus::Unlocked, &many_views(3), false);
    assert_eq!((model.rows.len(), model.hidden_accounts), (3, 0));
}

#[test]
fn locked_or_empty_shows_only_an_explanation() {
    let locked = build_tray_model(VaultStatus::Locked, &[], false);
    assert!(locked.placeholder.unwrap().contains("unlock"));
    assert!(locked.rows.is_empty() && locked.countdown_text.is_none());
    let empty = build_tray_model(VaultStatus::Unlocked, &[], false);
    assert!(empty.placeholder.unwrap().contains("import"));
}

#[test]
fn the_structure_ignores_code_and_countdown_changes_so_the_menu_is_not_replaced() {
    let before = build_tray_model(VaultStatus::Unlocked, &[view("a", "A", "111111", 20)], false);
    let after = build_tray_model(VaultStatus::Unlocked, &[view("a", "A", "999999", 19)], false);
    assert_ne!(before, after);
    assert_eq!(before.structure(), after.structure());
}

#[test]
fn the_structure_changes_when_accounts_lock_state_or_autostart_change() {
    let one = build_tray_model(VaultStatus::Unlocked, &[view("a", "A", "111111", 20)], false);
    let two = build_tray_model(VaultStatus::Unlocked, &[view("a", "A", "111111", 20), view("b", "B", "2", 20)], false);
    let locked = build_tray_model(VaultStatus::Locked, &[], false);
    let autostart = build_tray_model(VaultStatus::Unlocked, &[view("a", "A", "111111", 20)], true);
    for other in [&two, &locked, &autostart] {
        assert_ne!(one.structure(), other.structure());
    }
}

#[test]
fn the_structure_changes_when_accounts_are_reordered() {
    let first = build_tray_model(VaultStatus::Unlocked, &[view("a", "A", "1", 5), view("b", "B", "2", 5)], false);
    let swapped = build_tray_model(VaultStatus::Unlocked, &[view("b", "B", "2", 5), view("a", "A", "1", 5)], false);
    assert_ne!(first.structure(), swapped.structure());
}
