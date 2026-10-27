//! A TOTP seed that wipes itself from memory when dropped and never prints itself.

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Serialises as the plain byte list so existing vaults keep opening.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(Vec<u8>);

impl Secret {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Overwrites the bytes with zeros and empties the secret.
    pub fn wipe(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.wipe();
    }
}

/// Debug output ends up in panics and test failures, so the bytes are never shown.
impl std::fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Secret(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_never_contains_the_bytes() {
        let secret = Secret::new(vec![1, 2, 3]);
        assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
    }

    #[test]
    fn wiping_empties_the_secret() {
        let mut secret = Secret::new(vec![9; 20]);
        secret.wipe();
        assert!(secret.as_bytes().is_empty());
    }

    #[test]
    fn serialises_as_a_plain_byte_list_for_vault_compatibility() {
        let secret = Secret::new(vec![1, 2, 255]);
        assert_eq!(serde_json::to_string(&secret).unwrap(), "[1,2,255]");
        assert_eq!(serde_json::from_str::<Secret>("[1,2,255]").unwrap(), secret);
    }

    #[test]
    fn clones_compare_equal_and_are_independent() {
        let original = Secret::new(vec![5, 6]);
        let mut copy = original.clone();
        assert_eq!(copy, original);
        copy.wipe();
        assert_eq!(original.as_bytes(), &[5, 6]);
    }
}
