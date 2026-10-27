//! Deciding which imported accounts are genuinely new. Pure.

use crate::otp::Account;

/// Two accounts with the same secret produce the same codes, so they are the
/// same credential even if their labels differ.
fn already_has_secret(known: &[Account], candidate: &Account) -> bool {
    known.iter().any(|account| account.secret == candidate.secret)
}

/// Splits `imported` into the accounts to add and a count of those skipped
/// because the vault already holds them, or because the batch repeats them.
pub fn split_new_accounts(existing: &[Account], imported: Vec<Account>) -> (Vec<Account>, usize) {
    let mut new_accounts: Vec<Account> = Vec::new();
    let mut skipped_count = 0;
    for candidate in imported {
        let is_duplicate = already_has_secret(existing, &candidate) || already_has_secret(&new_accounts, &candidate);
        if is_duplicate {
            skipped_count += 1;
        } else {
            new_accounts.push(candidate);
        }
    }
    (new_accounts, skipped_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::Algorithm;

    fn account(name: &str, secret: &[u8]) -> Account {
        Account {
            issuer: String::new(),
            name: name.into(),
            secret: secret.to_vec(),
            algorithm: Algorithm::Sha1,
            digits: 6,
            period_seconds: 30,
        }
    }

    #[test]
    fn keeps_everything_when_nothing_overlaps() {
        let (added, skipped) = split_new_accounts(&[account("a", b"1")], vec![account("b", b"2")]);
        assert_eq!((added.len(), skipped), (1, 0));
    }

    #[test]
    fn skips_accounts_already_in_the_vault_even_if_renamed() {
        let (added, skipped) = split_new_accounts(&[account("old name", b"1")], vec![account("new name", b"1")]);
        assert_eq!((added.len(), skipped), (0, 1));
    }

    #[test]
    fn skips_repeats_within_one_batch() {
        let batch = vec![account("a", b"1"), account("a again", b"1"), account("b", b"2")];
        let (added, skipped) = split_new_accounts(&[], batch);
        assert_eq!((added.len(), skipped), (2, 1));
        assert_eq!(added[0].name, "a");
    }
}
