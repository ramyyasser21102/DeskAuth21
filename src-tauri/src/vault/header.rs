//! Fixed-size vault file header.
//!
//! ```text
//! "DA21" | version u8 | memory_kib u32 | passes u32 | lanes u32 | salt[16] | nonce[24]
//! ```
//!
//! All integers are big-endian. Everything before the nonce is authenticated
//! as AEAD associated data, so editing the salt or KDF costs breaks decryption.

use super::{KdfParams, VaultError};

const MAGIC: &[u8; 4] = b"DA21";
const FORMAT_VERSION: u8 = 1;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 24;

const VERSION_OFFSET: usize = MAGIC.len();
const MEMORY_OFFSET: usize = VERSION_OFFSET + 1;
const PASSES_OFFSET: usize = MEMORY_OFFSET + U32_LEN;
const LANES_OFFSET: usize = PASSES_OFFSET + U32_LEN;
const SALT_OFFSET: usize = LANES_OFFSET + U32_LEN;
const NONCE_OFFSET: usize = SALT_OFFSET + SALT_LEN;
const U32_LEN: usize = 4;

pub const HEADER_LEN: usize = NONCE_OFFSET + NONCE_LEN;

/// Upper bounds so a hostile file cannot make unlocking allocate gigabytes or spin for
/// minutes. They sit well above the defaults (64 MiB, 3 passes, 1 lane) to leave room
/// for stronger settings later.
const MAX_MEMORY_KIB: u32 = 256 * 1024;
const MAX_PASSES: u32 = 10;
const MAX_LANES: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub params: KdfParams,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}

impl Header {
    pub fn encode(&self) -> [u8; HEADER_LEN] {
        let mut bytes = [0u8; HEADER_LEN];
        bytes[..MAGIC.len()].copy_from_slice(MAGIC);
        bytes[VERSION_OFFSET] = FORMAT_VERSION;
        write_u32(&mut bytes, MEMORY_OFFSET, self.params.memory_kib);
        write_u32(&mut bytes, PASSES_OFFSET, self.params.passes);
        write_u32(&mut bytes, LANES_OFFSET, self.params.lanes);
        bytes[SALT_OFFSET..NONCE_OFFSET].copy_from_slice(&self.salt);
        bytes[NONCE_OFFSET..].copy_from_slice(&self.nonce);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, VaultError> {
        let is_recognised = bytes.len() >= HEADER_LEN
            && bytes.starts_with(MAGIC)
            && bytes[VERSION_OFFSET] == FORMAT_VERSION;
        if !is_recognised {
            return Err(VaultError::BadFormat);
        }
        let params = KdfParams {
            memory_kib: read_u32(bytes, MEMORY_OFFSET),
            passes: read_u32(bytes, PASSES_OFFSET),
            lanes: read_u32(bytes, LANES_OFFSET),
        };
        ensure_within_limits(params)?;
        Ok(Self {
            params,
            salt: read_array(bytes, SALT_OFFSET),
            nonce: read_array(bytes, NONCE_OFFSET),
        })
    }
}

/// The header bytes bound to the ciphertext (everything except the nonce, which
/// the cipher authenticates itself).
pub fn associated_data(encoded: &[u8]) -> &[u8] {
    &encoded[..NONCE_OFFSET]
}

pub fn ensure_within_limits(params: KdfParams) -> Result<(), VaultError> {
    let is_within_limits = params.memory_kib <= MAX_MEMORY_KIB
        && params.passes <= MAX_PASSES
        && params.lanes <= MAX_LANES;
    if is_within_limits { Ok(()) } else { Err(VaultError::BadParams) }
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + U32_LEN].copy_from_slice(&value.to_be_bytes());
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(read_array(bytes, offset))
}

/// Callers have already checked `bytes.len() >= HEADER_LEN`.
fn read_array<const LEN: usize>(bytes: &[u8], offset: usize) -> [u8; LEN] {
    bytes[offset..offset + LEN].try_into().expect("slice has exactly LEN bytes")
}
