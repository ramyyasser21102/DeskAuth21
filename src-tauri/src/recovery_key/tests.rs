use super::*;

#[test]
fn a_generated_key_has_24_words_from_the_list_and_is_accepted_back() {
    let key = generate().unwrap();
    assert_eq!(key.split(' ').count(), RECOVERY_WORD_COUNT);
    assert_eq!(*normalize(&key).unwrap(), *key);
}

#[test]
fn two_generated_keys_differ() {
    assert_ne!(*generate().unwrap(), *generate().unwrap());
}

#[test]
fn typing_in_capitals_with_odd_spacing_and_line_breaks_is_accepted() {
    let key = generate().unwrap();
    let messy = key.to_uppercase().replace(' ', "  \n ");
    assert_eq!(*normalize(&format!("  {messy}  ")).unwrap(), *key);
}

#[test]
fn the_wrong_number_of_words_is_reported_with_the_count() {
    let key = generate().unwrap();
    let short = key.split(' ').take(23).collect::<Vec<_>>().join(" ");
    assert_eq!(normalize(&short), Err(RecoveryKeyError::WrongWordCount(23)));
    assert_eq!(normalize(""), Err(RecoveryKeyError::WrongWordCount(0)));
}

#[test]
fn a_word_that_is_not_in_the_list_is_named() {
    let key = generate().unwrap();
    let mut typed: Vec<&str> = key.split(' ').collect();
    typed[5] = "cabage";
    assert_eq!(normalize(&typed.join(" ")), Err(RecoveryKeyError::UnknownWord("cabage".into())));
}
