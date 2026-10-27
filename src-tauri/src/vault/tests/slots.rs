use super::*;
use crate::vault::crypto::{derive_key, random_key};
use crate::vault::header::SALT_LEN;
use crate::vault::passphrase_slot::{KeyCost, PassphraseSlot};
use crate::vault::slot::{Slot, KIND_PASSPHRASE};

const SALT: [u8; SALT_LEN] = [5; SALT_LEN];

fn wrapped_with(passphrase: &str) -> (PassphraseSlot, crate::vault::crypto::Key) {
    let vault_key = random_key().unwrap();
    let kek = derive_key(passphrase, &SALT, FAST_PARAMS).unwrap();
    (PassphraseSlot::wrap(KIND_PASSPHRASE, &vault_key, &kek, KeyCost { salt: SALT, params: FAST_PARAMS }).unwrap(), vault_key)
}

#[test]
fn a_slot_returns_the_vault_key_for_the_right_passphrase_only() {
    let (slot, vault_key) = wrapped_with("right");
    assert_eq!(*slot.unwrap_with_passphrase("right").unwrap(), *vault_key);
    assert!(matches!(slot.unwrap_with_passphrase("wrong"), Err(VaultError::Decrypt)));
}

#[test]
fn a_slot_survives_encoding_and_decoding() {
    let (slot, _) = wrapped_with("pw");
    let body = Slot::Passphrase(slot.clone()).encode_body();
    assert_eq!(Slot::decode(1, &body).unwrap(), Slot::Passphrase(slot));
}

#[test]
fn a_slot_of_the_wrong_length_is_refused() {
    let (slot, _) = wrapped_with("pw");
    let body = Slot::Passphrase(slot).encode_body();
    assert!(matches!(Slot::decode(1, &body[1..]), Err(VaultError::BadFormat)));
}

#[test]
fn changing_the_recorded_cost_breaks_the_slot() {
    let (slot, _) = wrapped_with("pw");
    let mut body = Slot::Passphrase(slot).encode_body();
    body[3] ^= 1;
    let Slot::Passphrase(edited) = Slot::decode(1, &body).unwrap() else { panic!("still a passphrase slot") };
    assert!(edited.unwrap_with_passphrase("pw").is_err());
}

#[test]
fn a_slot_with_a_hostile_cost_is_refused_on_decoding() {
    let (slot, _) = wrapped_with("pw");
    let mut body = Slot::Passphrase(slot).encode_body();
    body[..4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(matches!(Slot::decode(1, &body), Err(VaultError::BadParams)));
}
