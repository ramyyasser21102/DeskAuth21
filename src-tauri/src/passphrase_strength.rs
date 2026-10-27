//! How hard a passphrase would be to guess, from the zxcvbn estimator. Advisory only:
//! the length rule in `passphrase.rs` is what is enforced.

use serde::Serialize;
use zxcvbn::{zxcvbn, Score};

/// The estimator slows down on very long input, and anything this long is already strong.
const MAX_CHARS_ANALYSED: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PassphraseStrength {
    /// 0 (very weak) to 4 (strong).
    pub score: u8,
    /// The estimator's most useful warning or suggestion, if it has one.
    pub feedback: Option<String>,
}

fn score_number(score: Score) -> u8 {
    match score {
        Score::Zero => 0,
        Score::One => 1,
        Score::Two => 2,
        Score::Three => 3,
        _ => 4,
    }
}

pub fn assess(passphrase: &str) -> PassphraseStrength {
    if passphrase.is_empty() {
        return PassphraseStrength { score: 0, feedback: None };
    }
    let analysed: String = passphrase.chars().take(MAX_CHARS_ANALYSED).collect();
    let result = zxcvbn(&analysed, &[]);
    let feedback = result.feedback().and_then(|feedback| {
        feedback.warning().map(|warning| warning.to_string()).or_else(|| feedback.suggestions().first().map(ToString::to_string))
    });
    PassphraseStrength { score: score_number(result.score()), feedback }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_typed_is_the_weakest_with_no_advice() {
        assert_eq!(assess(""), PassphraseStrength { score: 0, feedback: None });
    }

    #[test]
    fn a_common_password_scores_low_and_says_why() {
        let result = assess("password1234");
        assert!(result.score <= 1);
        assert!(result.feedback.is_some());
    }

    #[test]
    fn a_long_random_word_phrase_scores_high() {
        assert_eq!(assess("cabbage-unfasten-grant-sprawl-pastime-lantern").score, 4);
    }

    #[test]
    fn very_long_input_is_still_answered() {
        assert!(assess(&"x7Q!".repeat(500)).score <= 4);
    }
}
