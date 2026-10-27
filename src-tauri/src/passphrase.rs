//! Rules for choosing a new vault passphrase.

use crate::error::CommandError;

/// Long enough that guessing a stolen vault offline is impractical for a passphrase
/// of ordinary words. Existing vaults made with fewer characters still open.
pub const MIN_PASSPHRASE_CHARS: usize = 12;

pub fn validate_new_passphrase(passphrase: &str) -> Result<(), CommandError> {
    if passphrase.chars().count() < MIN_PASSPHRASE_CHARS {
        return Err(CommandError::PassphraseTooShort(MIN_PASSPHRASE_CHARS));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_characters_not_bytes() {
        assert!(validate_new_passphrase("12345678901").is_err());
        assert!(validate_new_passphrase("123456789012").is_ok());
        // 12 characters, 24 bytes
        assert!(validate_new_passphrase("éééééééééééé").is_ok());
    }
}
