//! The version 2 file: a header of unlock slots, then the encrypted data.
//!
//! ```text
//! "DA21" | version u8 = 2 | slot_count u8 | slot... | payload_nonce[24] | ciphertext
//! slot = kind u8 | body_len u16 (big-endian) | body
//! ```
//!
//! Everything before the payload nonce is authenticated as associated data, so
//! removing or editing a slot makes the data fail to open.

use super::crypto::NONCE_LEN;
use super::slot::Slot;
use super::VaultError;

pub const MAGIC: &[u8; 4] = b"DA21";
pub const VERSION_1: u8 = 1;
pub const VERSION_2: u8 = 2;
const PREAMBLE_LEN: usize = MAGIC.len() + 2;
const SLOT_HEADER_LEN: usize = 3;
/// A hostile file must not make us loop or allocate without bound.
const MAX_SLOTS: usize = 8;
const MAX_SLOT_BODY: usize = 512;

pub struct ParsedFile<'a> {
    pub slots: Vec<Slot>,
    /// The header bytes the data is bound to.
    pub associated_data: &'a [u8],
    pub payload_nonce: [u8; NONCE_LEN],
    pub ciphertext: &'a [u8],
}

pub fn file_version(bytes: &[u8]) -> Option<u8> {
    bytes.starts_with(MAGIC).then(|| bytes.get(MAGIC.len()).copied()).flatten()
}

pub fn header_bytes(slots: &[Slot]) -> Vec<u8> {
    let slot_count = u8::try_from(slots.len()).expect("a vault never holds more than MAX_SLOTS slots");
    let mut bytes = [MAGIC.as_slice(), &[VERSION_2, slot_count]].concat();
    for slot in slots {
        let body = slot.encode_body();
        let body_len = u16::try_from(body.len()).expect("slot bodies are far below 64 KiB");
        bytes.extend_from_slice(&[slot.kind()]);
        bytes.extend_from_slice(&body_len.to_be_bytes());
        bytes.extend_from_slice(&body);
    }
    bytes
}

pub fn parse(bytes: &[u8]) -> Result<ParsedFile<'_>, VaultError> {
    if file_version(bytes) != Some(VERSION_2) || bytes.len() < PREAMBLE_LEN {
        return Err(VaultError::BadFormat);
    }
    let slot_count = usize::from(bytes[MAGIC.len() + 1]);
    if slot_count == 0 || slot_count > MAX_SLOTS {
        return Err(VaultError::BadFormat);
    }
    let mut position = PREAMBLE_LEN;
    let mut slots = Vec::with_capacity(slot_count);
    for _ in 0..slot_count {
        let (slot, next) = parse_slot(bytes, position)?;
        slots.push(slot);
        position = next;
    }
    let after_header = position + NONCE_LEN;
    let nonce = bytes.get(position..after_header).ok_or(VaultError::BadFormat)?;
    Ok(ParsedFile {
        slots,
        associated_data: &bytes[..position],
        payload_nonce: nonce.try_into().map_err(|_| VaultError::BadFormat)?,
        ciphertext: &bytes[after_header..],
    })
}

fn parse_slot(bytes: &[u8], position: usize) -> Result<(Slot, usize), VaultError> {
    let header = bytes.get(position..position + SLOT_HEADER_LEN).ok_or(VaultError::BadFormat)?;
    let body_len = usize::from(u16::from_be_bytes([header[1], header[2]]));
    if body_len > MAX_SLOT_BODY {
        return Err(VaultError::BadFormat);
    }
    let body_start = position + SLOT_HEADER_LEN;
    let body = bytes.get(body_start..body_start + body_len).ok_or(VaultError::BadFormat)?;
    Ok((Slot::decode(header[0], body)?, body_start + body_len))
}
