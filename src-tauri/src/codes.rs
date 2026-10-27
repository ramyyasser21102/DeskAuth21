//! Pure presentation of one-time codes for the window and the tray.

use crate::otp::Account;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodeView {
    pub label: String,
    pub code: String,
    pub seconds_left: u64,
}

pub fn build_code_views(accounts: &[Account], unix_seconds: u64) -> Vec<CodeView> {
    accounts
        .iter()
        .map(|account| CodeView {
            label: account.display_label(),
            code: account.code_at(unix_seconds),
            seconds_left: account.seconds_until_next_code(unix_seconds),
        })
        .collect()
}

/// "123456" becomes "123 456" so it is easier to read and type.
pub fn split_code_in_half(code: &str) -> String {
    let (first_half, second_half) = code.split_at(code.len() / 2);
    format!("{first_half} {second_half}")
}

pub fn tray_menu_text(view: &CodeView) -> String {
    let code = split_code_in_half(&view.code);
    format!("{}  {}  ({}s)", view.label, code, view.seconds_left)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::Algorithm;

    fn sample_account() -> Account {
        Account {
            issuer: "Acme".into(),
            name: "a@b".into(),
            secret: b"12345678901234567890".to_vec(),
            algorithm: Algorithm::Sha1,
            digits: 6,
            period_seconds: 30,
        }
    }

    #[test]
    fn builds_a_view_per_account_at_the_given_time() {
        let views = build_code_views(&[sample_account()], 59);
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].label, "Acme (a@b)");
        assert_eq!(views[0].code, sample_account().code_at(59));
        assert_eq!(views[0].seconds_left, 1);
    }

    #[test]
    fn splits_codes_for_readability() {
        assert_eq!(split_code_in_half("123456"), "123 456");
        assert_eq!(split_code_in_half("12345678"), "1234 5678");
    }

    #[test]
    fn formats_tray_text() {
        let view = CodeView { label: "Acme".into(), code: "123456".into(), seconds_left: 7 };
        assert_eq!(tray_menu_text(&view), "Acme  123 456  (7s)");
    }
}
