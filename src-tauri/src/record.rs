//! What the vault stores: accounts with stable identities, plus tombstones for
//! deleted ones, so deletions and renames can be merged between devices.

use crate::otp::Account;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const ID_DOMAIN: &[u8] = b"deskauth21/account-id/v1";
const ID_HEX_CHARS: usize = 32;

/// The same secret always gives the same id, on every device, with no
/// coordination. It only ever exists inside the encrypted vault.
pub fn account_id_for(secret: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ID_DOMAIN);
    hasher.update(secret);
    let hex: String = hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
    hex[..ID_HEX_CHARS].to_string()
}

/// An account plus when it was last changed (milliseconds since the Unix epoch).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: String,
    pub updated_at_ms: u64,
    #[serde(flatten)]
    pub account: Account,
}

impl AccountRecord {
    pub fn new(account: Account, now_ms: u64) -> Self {
        Self { id: account_id_for(&account.secret), updated_at_ms: now_ms, account }
    }
}

/// Remembers that an account was deleted, so another device that still has it
/// does not bring it back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tombstone {
    pub id: String,
    pub deleted_at_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultData {
    pub accounts: Vec<AccountRecord>,
    #[serde(default)]
    pub tombstones: Vec<Tombstone>,
}

/// Vaults written before accounts had identities hold a bare list of accounts.
#[derive(Deserialize)]
#[serde(untagged)]
enum StoredVaultData {
    Current(VaultData),
    Legacy(Vec<Account>),
}

/// Reads the current format, or upgrades the old one: legacy accounts get
/// their secret-derived id and a timestamp of 0, so any real edit supersedes them.
pub fn parse_vault_data(json: &[u8]) -> Result<VaultData, serde_json::Error> {
    Ok(match serde_json::from_slice(json)? {
        StoredVaultData::Current(data) => data,
        StoredVaultData::Legacy(accounts) => VaultData {
            accounts: accounts.into_iter().map(|account| AccountRecord::new(account, 0)).collect(),
            tombstones: Vec::new(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::Algorithm;

    fn account(secret: &[u8]) -> Account {
        Account {
            issuer: "Acme".into(),
            name: "a@b".into(),
            secret: secret.to_vec(),
            algorithm: Algorithm::Sha1,
            digits: 6,
            period_seconds: 30,
        }
    }

    #[test]
    fn the_same_secret_always_has_the_same_id() {
        assert_eq!(account_id_for(b"seed"), account_id_for(b"seed"));
        assert_ne!(account_id_for(b"seed"), account_id_for(b"other"));
        assert_eq!(account_id_for(b"seed").len(), ID_HEX_CHARS);
    }

    #[test]
    fn an_id_does_not_contain_the_secret() {
        let id = account_id_for(b"supersecretvalue");
        assert!(!id.contains("supersecret"));
    }

    #[test]
    fn current_data_round_trips() {
        let data = VaultData {
            accounts: vec![AccountRecord::new(account(b"one"), 100)],
            tombstones: vec![Tombstone { id: "gone".into(), deleted_at_ms: 50 }],
        };
        let json = serde_json::to_vec(&data).unwrap();
        assert_eq!(parse_vault_data(&json).unwrap(), data);
    }

    #[test]
    fn a_legacy_list_of_accounts_is_upgraded_with_derived_ids() {
        let legacy = serde_json::to_vec(&vec![account(b"one"), account(b"two")]).unwrap();
        let data = parse_vault_data(&legacy).unwrap();
        assert_eq!(data.accounts.len(), 2);
        assert_eq!(data.accounts[0].id, account_id_for(b"one"));
        assert_eq!(data.accounts[0].updated_at_ms, 0);
        assert!(data.tombstones.is_empty());
    }

    #[test]
    fn two_devices_upgrading_the_same_legacy_vault_agree_on_every_id() {
        let legacy = serde_json::to_vec(&vec![account(b"one")]).unwrap();
        assert_eq!(parse_vault_data(&legacy).unwrap(), parse_vault_data(&legacy).unwrap());
    }

    #[test]
    fn rejects_data_that_is_neither_format() {
        assert!(parse_vault_data(b"{\"nope\":1}").is_err());
        assert!(parse_vault_data(b"garbage").is_err());
    }
}
