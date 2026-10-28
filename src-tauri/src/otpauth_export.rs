//! Turns accounts back into standard `otpauth://` links, one per line, readable by
//! DA21's own import and by other authenticators. Pure.

use crate::otp::Algorithm;
use crate::record::AccountRecord;
use zeroize::Zeroizing;

const BASE32_ALPHABET: base32::Alphabet = base32::Alphabet::Rfc4648 { padding: false };

const fn algorithm_name(algorithm: Algorithm) -> &'static str {
    match algorithm {
        Algorithm::Sha1 => "SHA1",
        Algorithm::Sha256 => "SHA256",
        Algorithm::Sha512 => "SHA512",
    }
}

fn otpauth_link(record: &AccountRecord) -> String {
    let account = &record.account;
    let label = if account.issuer.is_empty() {
        urlencoding::encode(&account.name).into_owned()
    } else {
        format!("{}:{}", urlencoding::encode(&account.issuer), urlencoding::encode(&account.name))
    };
    let secret = base32::encode(BASE32_ALPHABET, account.secret.as_bytes());
    let issuer = if account.issuer.is_empty() { String::new() } else { format!("&issuer={}", urlencoding::encode(&account.issuer)) };
    format!(
        "otpauth://totp/{label}?secret={secret}{issuer}&algorithm={}&digits={}&period={}",
        algorithm_name(account.algorithm),
        account.digits,
        account.period_seconds,
    )
}

/// The text wipes itself when dropped, because it holds every seed in the open.
pub fn export_otpauth_lines(records: &[AccountRecord]) -> Zeroizing<String> {
    let mut text = Zeroizing::new(String::new());
    for record in records {
        text.push_str(&otpauth_link(record));
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::parse_otpauth_uri;
    use crate::otp::Account;
    use crate::secret::Secret;

    fn record(issuer: &str, name: &str, algorithm: Algorithm) -> AccountRecord {
        let account = Account { issuer: issuer.into(), name: name.into(), secret: Secret::new(b"12345678901234567890".to_vec()), algorithm, digits: 8, period_seconds: 60 };
        AccountRecord::new(account, 1)
    }

    #[test]
    fn every_account_comes_back_the_same_through_import() {
        let originals = [record("Acme Co", "a.b+c@example.com", Algorithm::Sha256), record("", "plain name", Algorithm::Sha512)];
        for original in originals {
            let text = export_otpauth_lines(std::slice::from_ref(&original));
            let parsed = parse_otpauth_uri(text.trim()).unwrap();
            assert_eq!(parsed, original.account);
        }
    }

    #[test]
    fn writes_one_link_per_line() {
        let text = export_otpauth_lines(&[record("A", "x", Algorithm::Sha1), record("B", "y", Algorithm::Sha1)]);
        assert_eq!(text.lines().count(), 2);
        assert!(text.lines().all(|line| line.starts_with("otpauth://totp/")));
    }

    #[test]
    fn no_accounts_gives_no_text() {
        assert!(export_otpauth_lines(&[]).is_empty());
    }
}
