use super::*;
use crate::vault::crypto::random_key;
use crate::vault::hardware_slots::{truncate_label, EnvelopeSlot, HardwareSlot, KeyIdentity, HMAC_SALT_LEN, MAX_LABEL_BYTES};
use crate::vault::slot::Slot;

const OUTPUT: [u8; 32] = [7; 32];
const SALT: [u8; HMAC_SALT_LEN] = [3; HMAC_SALT_LEN];

fn hardware_slot() -> (HardwareSlot, crate::vault::crypto::Key) {
    let shared = random_key().unwrap();
    (HardwareSlot::wrap(&shared, &OUTPUT, KeyIdentity { credential_id: vec![9, 8, 7], salt: SALT, label: "My key".to_string() }).unwrap(), shared)
}

#[test]
fn a_hardware_slot_returns_the_shared_secret_only_for_the_right_key_output() {
    let (slot, shared) = hardware_slot();
    assert_eq!(*slot.unwrap_with(&OUTPUT).unwrap(), *shared);
    assert!(matches!(slot.unwrap_with(&[8; 32]), Err(VaultError::Decrypt)));
}

#[test]
fn a_hardware_slot_survives_encoding_and_decoding() {
    let (slot, _) = hardware_slot();
    let body = Slot::Hardware(slot.clone()).encode_body();
    assert_eq!(Slot::decode(3, &body).unwrap(), Slot::Hardware(slot));
}

#[test]
fn every_truncation_of_a_hardware_slot_is_refused_without_panicking() {
    let (slot, _) = hardware_slot();
    let body = Slot::Hardware(slot).encode_body();
    for length in 0..body.len() {
        assert!(Slot::decode(3, &body[..length]).is_err(), "length {length}");
    }
}

#[test]
fn trailing_bytes_after_a_hardware_slot_are_refused() {
    let (slot, _) = hardware_slot();
    let mut body = Slot::Hardware(slot).encode_body();
    body.push(0);
    assert!(Slot::decode(3, &body).is_err());
}

#[test]
fn an_envelope_needs_both_halves_and_survives_encoding() {
    let (vault_key, gate, shared) = (random_key().unwrap(), random_key().unwrap(), random_key().unwrap());
    let envelope = EnvelopeSlot::wrap(&vault_key, &gate, &shared).unwrap();
    assert_eq!(*envelope.unwrap_with(&gate, &shared).unwrap(), *vault_key);
    assert!(envelope.unwrap_with(&gate, &random_key().unwrap()).is_err());
    assert!(envelope.unwrap_with(&random_key().unwrap(), &shared).is_err());
    let body = Slot::Envelope(envelope.clone()).encode_body();
    assert_eq!(Slot::decode(5, &body).unwrap(), Slot::Envelope(envelope));
}

#[test]
fn a_label_is_cut_to_the_limit_without_splitting_a_character() {
    assert_eq!(truncate_label("short"), "short");
    let long = "é".repeat(MAX_LABEL_BYTES);
    let cut = truncate_label(&long);
    assert!(cut.len() <= MAX_LABEL_BYTES);
    assert!(cut.chars().all(|letter| letter == 'é'));
}
