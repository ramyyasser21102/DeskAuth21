use super::*;
use crate::vault::layout::{header_bytes, parse};
use crate::vault::slot::Slot;
use crate::vault::slotted::SlottedKeys;

const UNKNOWN_KIND: u8 = 99;

#[test]
fn a_header_survives_a_round_trip_through_the_parser() {
    let bytes = sealed_vault("pw");
    let parsed = parse(&bytes).unwrap();
    assert_eq!(header_bytes(&parsed.slots), parsed.associated_data);
    assert_eq!(parsed.slots.len(), 1);
}

#[test]
fn every_truncated_file_is_refused_without_panicking() {
    let bytes = sealed_vault("pw");
    for length in 0..bytes.len() {
        assert!(Session::open(&bytes[..length], "pw").is_err(), "length {length}");
    }
}

#[test]
fn a_slot_count_of_zero_or_beyond_the_limit_is_refused() {
    let mut bytes = sealed_vault("pw");
    bytes[5] = 0;
    assert!(matches!(parse(&bytes), Err(VaultError::BadFormat)));
    bytes[5] = 200;
    assert!(matches!(parse(&bytes), Err(VaultError::BadFormat)));
}

#[test]
fn a_slot_claiming_a_huge_body_is_refused() {
    let mut bytes = sealed_vault("pw");
    bytes[7] = 0xff;
    bytes[8] = 0xff;
    assert!(matches!(parse(&bytes), Err(VaultError::BadFormat)));
}

#[test]
fn a_slot_kind_this_version_does_not_know_is_kept_when_the_vault_is_saved() {
    let mut keys = SlottedKeys::create("pw", FAST_PARAMS).unwrap();
    keys.slots.push(Slot::Unknown { kind: UNKNOWN_KIND, body: vec![1, 2, 3] });
    let file = keys.seal(&sample_data()).unwrap();
    let (reopened, data) = SlottedKeys::open(&file, "pw", &crate::key_device::NoKeySupport, None).unwrap();
    assert_eq!(data, sample_data());
    assert!(reopened.slots.contains(&Slot::Unknown { kind: UNKNOWN_KIND, body: vec![1, 2, 3] }));
    let resaved = reopened.seal(&data).unwrap();
    assert!(parse(&resaved).unwrap().slots.contains(&Slot::Unknown { kind: UNKNOWN_KIND, body: vec![1, 2, 3] }));
}
