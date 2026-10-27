//! Makes a random passphrase from six words of the EFF large word list (7,776 words,
//! about 77 bits). Randomness is passed in, so the choosing is testable.

const WORD_LIST: &str = include_str!("../assets/eff_large_wordlist.txt");
const WORD_COUNT: usize = 6;
const SEPARATOR: &str = "-";

/// Each line of the list is a five-dice roll, a tab, then the word.
fn words() -> Vec<&'static str> {
    WORD_LIST.lines().filter_map(|line| line.split_once('\t').map(|(_, word)| word)).collect()
}

/// A fair pick in `0..limit`. Taking a random number modulo `limit` would favour the
/// low indexes, so numbers from the uneven top slice are thrown away and redrawn.
pub fn index_below(limit: u32, mut next_random: impl FnMut() -> u32) -> u32 {
    let unfair_from = u32::MAX - (u32::MAX % limit);
    loop {
        let candidate = next_random();
        if candidate < unfair_from {
            return candidate % limit;
        }
    }
}

pub fn passphrase_from(words: &[&str], mut next_random: impl FnMut() -> u32) -> String {
    let limit = u32::try_from(words.len()).expect("the word list is far smaller than u32::MAX");
    let chosen: Vec<&str> = (0..WORD_COUNT).map(|_| words[index_below(limit, &mut next_random) as usize]).collect();
    chosen.join(SEPARATOR)
}

pub fn generate_passphrase() -> Result<String, getrandom::Error> {
    let mut failure = None;
    let phrase = passphrase_from(&words(), || {
        let mut bytes = [0u8; 4];
        if let Err(error) = getrandom::fill(&mut bytes) {
            failure = Some(error);
        }
        u32::from_le_bytes(bytes)
    });
    failure.map_or(Ok(phrase), Err)
}

#[cfg(test)]
mod tests;
