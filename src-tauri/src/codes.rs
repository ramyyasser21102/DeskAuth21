//! Pure presentation of one-time codes for the window and the tray.

use crate::record::AccountRecord;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodeView {
    /// Stable across devices and reordering; the UI refers to accounts by this.
    pub id: String,
    pub label: String,
    pub issuer: String,
    pub name: String,
    pub code: String,
    pub seconds_left: u64,
}

pub fn build_code_views(records: &[AccountRecord], unix_seconds: u64) -> Vec<CodeView> {
    records
        .iter()
        .map(|record| CodeView {
            id: record.id.clone(),
            label: record.account.display_label(),
            issuer: record.account.issuer.clone(),
            name: record.account.name.clone(),
            code: record.account.code_at(unix_seconds),
            seconds_left: record.account.seconds_until_next_code(unix_seconds),
        })
        .collect()
}

/// "123456" becomes "123 456" so it is easier to read and type.
pub fn split_code_in_half(code: &str) -> String {
    let (first_half, second_half) = code.split_at(code.len() / 2);
    format!("{first_half} {second_half}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::{Account, Algorithm};

    fn sample_record() -> AccountRecord {
        let account = Account {
            issuer: "Acme".into(),
            name: "a@b".into(),
            secret: b"12345678901234567890".to_vec(),
            algorithm: Algorithm::Sha1,
            digits: 6,
            period_seconds: 30,
        };
        AccountRecord::new(account, 1)
    }

    #[test]
    fn builds_a_view_per_account_at_the_given_time() {
        let record = sample_record();
        let views = build_code_views(std::slice::from_ref(&record), 59);
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].id, record.id);
        assert_eq!(views[0].label, "Acme (a@b)");
        assert_eq!((views[0].issuer.as_str(), views[0].name.as_str()), ("Acme", "a@b"));
        assert_eq!(views[0].code, record.account.code_at(59));
        assert_eq!(views[0].seconds_left, 1);
    }

    #[test]
    fn splits_codes_for_readability() {
        assert_eq!(split_code_in_half("123456"), "123 456");
        assert_eq!(split_code_in_half("12345678"), "1234 5678");
    }
}
