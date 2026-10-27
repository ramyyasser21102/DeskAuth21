//! Encrypted on-disk vault: Argon2id key derivation + XChaCha20-Poly1305.
//!
//! Two file formats exist. Version 1 (`legacy`, `header`) encrypts straight under a
//! key made from the passphrase. Version 2 (`slotted`, `layout`, `slot`) encrypts under
//! a random vault key that unlock slots wrap, so ways of unlocking can be added and
//! changed without re-encrypting. `session` is the one type callers use.
//! The plaintext is the JSON-serialised vault contents.

mod crypto;
mod hardware_ops;
mod hardware_slots;
mod header;
mod layout;
mod legacy;
mod passphrase_slot;
mod session;
mod slot;
mod slotted;
#[cfg(test)]
mod tests;

pub use hardware_ops::{file_requires_hardware_key, HardwareKeyInfo};
pub use session::Session;

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("not a DA21 vault, or an unsupported version")]
    BadFormat,
    #[error("wrong passphrase or damaged vault file; check the passphrase and try again")]
    Decrypt,
    #[error("could not encrypt the vault")]
    Encrypt,
    #[error("invalid key derivation parameters")]
    BadParams,
    #[error("could not gather randomness from the OS")]
    Random,
    #[error(transparent)]
    Key(#[from] crate::key_device::KeyError),
    #[error("create a recovery key first, so losing the hardware key cannot lock you out")]
    NeedsRecoveryKey,
    #[error("a hardware key is already required for this vault")]
    HardwareKeyAlreadyOn,
    #[error("this vault does not require a hardware key")]
    HardwareKeyNotOn,
    #[error("lock and unlock the vault, then try again")]
    LockAndUnlockFirst,
    #[error("a vault needs at least one hardware key; turn the requirement off instead")]
    LastHardwareKey,
    #[error("that hardware key is not registered for this vault")]
    UnknownHardwareKey,
    #[error("upgrade the vault format first")]
    NeedsUpgrade,
    #[error("that recovery key does not open this vault")]
    WrongRecoveryKey,
    #[error("this vault was upgraded on another computer; lock and unlock to continue")]
    NewerFormat,
    #[error("this vault file is in the older format; update the other computer or lock and unlock")]
    OlderFormat,
    #[error("vault contents are not valid: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub memory_kib: u32,
    pub passes: u32,
    pub lanes: u32,
}

impl KdfParams {
    /// 128 MiB, 3 passes, 1 lane: about 0.13 s to unlock on a recent laptop. Vaults made
    /// with the earlier 64 MiB setting keep it, because the file records its own cost.
    pub const DEFAULT: Self = Self { memory_kib: 128 * 1024, passes: 3, lanes: 1 };
}
