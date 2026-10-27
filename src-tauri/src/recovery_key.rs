//! The recovery key: 24 random words that open the vault if the passphrase is lost.
//! About 310 bits, so it needs no slow key derivation. Words are separated by spaces
//! because some list words contain a hyphen.

use crate::passphrase_generator::{random_words, words};
use std::collections::HashSet;
use zeroize::Zeroizing;

pub const RECOVERY_WORD_COUNT: usize = 24;
const SEPARATOR: &str = " ";

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RecoveryKeyError {
    #[error("a recovery key has 24 words; this one has {0}")]
    WrongWordCount(usize),
    #[error("\"{0}\" is not one of the recovery key words; check its spelling")]
    UnknownWord(String),
}

pub fn generate() -> Result<Zeroizing<String>, getrandom::Error> {
    random_words(RECOVERY_WORD_COUNT, SEPARATOR).map(Zeroizing::new)
}

/// Accepts what a person typed or pasted: any case, any spacing or line breaks. Returns
/// the one canonical form that the key is made from, or says which word is wrong.
pub fn normalize(input: &str) -> Result<Zeroizing<String>, RecoveryKeyError> {
    normalize_against(input, &words())
}

fn normalize_against(input: &str, list: &[&str]) -> Result<Zeroizing<String>, RecoveryKeyError> {
    let typed: Vec<String> = input.split_whitespace().map(str::to_lowercase).collect();
    if typed.len() != RECOVERY_WORD_COUNT {
        return Err(RecoveryKeyError::WrongWordCount(typed.len()));
    }
    let known: HashSet<&str> = list.iter().copied().collect();
    if let Some(unknown) = typed.iter().find(|word| !known.contains(word.as_str())) {
        return Err(RecoveryKeyError::UnknownWord(unknown.clone()));
    }
    Ok(Zeroizing::new(typed.join(SEPARATOR)))
}

#[cfg(test)]
mod tests;
