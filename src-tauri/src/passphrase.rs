//! Rules for choosing a new vault passphrase.

use crate::error::CommandError;

pub const MIN_PASSPHRASE_CHARS: usize = 8;

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
        assert!(validate_new_passphrase("1234567").is_err());
        assert!(validate_new_passphrase("12345678").is_ok());
        // 8 characters, 16 bytes
        assert!(validate_new_passphrase("éééééééé").is_ok());
    }
}
