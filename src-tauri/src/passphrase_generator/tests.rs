use super::*;
use std::collections::HashSet;

#[test]
fn the_bundled_list_is_the_full_unique_dice_list() {
    let list = words();
    assert_eq!(list.len(), 7776);
    assert_eq!(list.iter().collect::<HashSet<_>>().len(), 7776);
    assert!(list.iter().all(|word| word.chars().all(|letter| letter.is_ascii_lowercase() || letter == '-')));
}

#[test]
fn it_builds_six_words_joined_by_hyphens() {
    let list = ["alpha", "bravo", "charlie"];
    let mut next = 0u32;
    let phrase = passphrase_from(&list, || {
        next += 1;
        next
    });
    assert_eq!(phrase.split('-').count(), 6);
    assert!(phrase.split('-').all(|word| list.contains(&word)));
}

#[test]
fn numbers_from_the_uneven_top_slice_are_redrawn() {
    // For a limit of 3, u32::MAX leaves a remainder of 0 only if divisible; use a limit
    // that does not divide evenly so the top slice is non-empty.
    let limit = 7776;
    let unfair_from = u32::MAX - (u32::MAX % limit);
    let mut drawn = vec![u32::MAX, unfair_from, 5].into_iter();
    assert_eq!(index_below(limit, || drawn.next().unwrap()), 5);
}

#[test]
fn every_index_is_reachable_and_in_range() {
    let mut value = 0u32;
    let seen: HashSet<u32> = (0..3000)
        .map(|_| {
            value = value.wrapping_add(1_000_003);
            index_below(10, || value)
        })
        .collect();
    assert!(seen.iter().all(|index| *index < 10));
    assert_eq!(seen.len(), 10);
}

#[test]
fn two_real_passphrases_differ() {
    let first = generate_passphrase().unwrap();
    let second = generate_passphrase().unwrap();
    assert_ne!(first, second);
    assert!(first.chars().count() >= 12);
}
