use super::*;

const LISTING: TrayOptions = TrayOptions { is_autostart_enabled: false, shows_codes: true, lists_accounts: true };
const SMALL_MENU: TrayOptions = TrayOptions { lists_accounts: false, ..LISTING };

fn view_for(id: &str, label: &str, vendor: Vendor) -> CodeView {
    CodeView {
        id: id.into(),
        label: label.into(),
        issuer: String::new(),
        name: label.into(),
        vendor,
        code: "123456".into(),
        seconds_left: 20,
        period_seconds: 30,
    }
}

fn view(id: &str, label: &str, code: &str, seconds_left: u64) -> CodeView {
    CodeView { code: code.into(), seconds_left, ..view_for(id, label, Vendor::Other) }
}

fn many_views(count: usize) -> Vec<CodeView> {
    (0..count).map(|index| view(&format!("id{index}"), &format!("Service {index}"), "123456", 20)).collect()
}

fn account_texts(model: &TrayModel) -> Vec<String> {
    model.account_rows().map(|row| row.text.clone()).collect()
}

fn headers(model: &TrayModel) -> Vec<String> {
    model.entries.iter().filter_map(|entry| if let TrayEntry::Header(text) = entry { Some(text.clone()) } else { None }).collect()
}

#[test]
fn every_account_is_listed_however_many_there_are() {
    let model = build_tray_model(VaultStatus::Unlocked, &many_views(40), LISTING);
    assert_eq!(model.account_rows().count(), 40);
}

#[test]
fn a_row_shows_the_full_name_and_the_code_in_two_halves_without_a_per_row_countdown() {
    let long = "GitHub (alice.smith@example-company-with-a-very-long-name.com)";
    let model = build_tray_model(VaultStatus::Unlocked, &[view("a", long, "123456", 9)], LISTING);
    assert_eq!(account_texts(&model), vec![format!("{long}   123 456")]);
}

#[test]
fn accounts_sit_under_a_heading_for_their_app_with_apps_a_to_z_and_other_last() {
    let views = [
        view_for("1", "Zoom", Vendor::Other),
        view_for("2", "Slack", Vendor::Authy),
        view_for("3", "Amazon", Vendor::Authy),
        view_for("4", "GitHub", Vendor::Aegis),
    ];
    let model = build_tray_model(VaultStatus::Unlocked, &views, TrayOptions { shows_codes: false, ..LISTING });
    let expected = [
        format!("{} (1)", Vendor::Aegis.display_name()),
        format!("{} (2)", Vendor::Authy.display_name()),
        format!("{} (1)", Vendor::Other.display_name()),
    ];
    assert_eq!(headers(&model), expected);
    assert_eq!(account_texts(&model), vec!["GitHub", "Amazon", "Slack", "Zoom"]);
}

#[test]
fn one_countdown_shows_the_soonest_expiry() {
    let views = [view("a", "A", "111111", 20), view("b", "B", "222222", 7)];
    assert_eq!(build_tray_model(VaultStatus::Unlocked, &views, LISTING).countdown_text.as_deref(), Some("Codes refresh in 7s"));
}

#[test]
fn hidden_codes_leave_only_the_name_and_no_countdown() {
    let options = TrayOptions { shows_codes: false, ..LISTING };
    let model = build_tray_model(VaultStatus::Unlocked, &[view("a", "Acme (a@b)", "123456", 9)], options);
    assert_eq!(account_texts(&model), vec!["Acme (a@b)"]);
    assert_eq!(model.countdown_text, None);
}

#[test]
fn locked_or_empty_shows_only_an_explanation() {
    let locked = build_tray_model(VaultStatus::Locked, &[], LISTING);
    assert!(locked.placeholder.unwrap().contains("unlock"));
    assert!(locked.entries.is_empty() && locked.countdown_text.is_none() && !locked.is_unlocked);
    let empty = build_tray_model(VaultStatus::Unlocked, &[], LISTING);
    assert!(empty.placeholder.unwrap().contains("import"));
}

#[test]
fn the_short_menu_lists_no_accounts_no_countdown_and_no_status_line() {
    let model = build_tray_model(VaultStatus::Unlocked, &many_views(5), SMALL_MENU);
    assert!(model.entries.is_empty() && model.countdown_text.is_none() && model.placeholder.is_none());
    assert!(!model.offers_panel && model.is_unlocked);
}

#[test]
fn the_listing_menu_offers_the_panel_the_short_menu_does_not() {
    assert!(build_tray_model(VaultStatus::Unlocked, &[], LISTING).offers_panel);
    assert!(!build_tray_model(VaultStatus::Unlocked, &[], SMALL_MENU).offers_panel);
}

#[test]
fn the_structure_ignores_code_and_countdown_changes_so_the_menu_is_not_replaced() {
    let before = build_tray_model(VaultStatus::Unlocked, &[view("a", "A", "111111", 20)], LISTING);
    let after = build_tray_model(VaultStatus::Unlocked, &[view("a", "A", "999999", 19)], LISTING);
    assert_ne!(before, after);
    assert_eq!(before.structure(), after.structure());
}

#[test]
fn the_structure_changes_when_accounts_lock_state_autostart_or_mode_change() {
    let one = [view("a", "A", "111111", 20)];
    let base = build_tray_model(VaultStatus::Unlocked, &one, LISTING);
    let two = build_tray_model(VaultStatus::Unlocked, &[one[0].clone(), view("b", "B", "2", 20)], LISTING);
    let locked = build_tray_model(VaultStatus::Locked, &[], LISTING);
    let autostart = build_tray_model(VaultStatus::Unlocked, &one, TrayOptions { is_autostart_enabled: true, ..LISTING });
    let short = build_tray_model(VaultStatus::Unlocked, &one, SMALL_MENU);
    for other in [&two, &locked, &autostart, &short] {
        assert_ne!(base.structure(), other.structure());
    }
}

#[test]
fn the_structure_changes_when_an_account_moves_to_another_app() {
    let before = build_tray_model(VaultStatus::Unlocked, &[view_for("a", "A", Vendor::Authy)], LISTING);
    let after = build_tray_model(VaultStatus::Unlocked, &[view_for("a", "A", Vendor::Aegis)], LISTING);
    assert_ne!(before.structure(), after.structure());
}

#[test]
fn switching_the_codes_on_or_off_changes_the_structure_so_the_menu_is_rebuilt() {
    let views = [view("a", "A", "111111", 20)];
    let shown = build_tray_model(VaultStatus::Unlocked, &views, LISTING);
    let hidden = build_tray_model(VaultStatus::Unlocked, &views, TrayOptions { shows_codes: false, ..LISTING });
    assert_ne!(shown.structure(), hidden.structure());
}
