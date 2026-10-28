//! Version 1: the data is encrypted straight under a key made from the passphrase.
//! New vaults no longer use it, but vaults made earlier still open and keep saving
//! in this format until they are upgraded.

use super::crypto::{decrypt, derive_key, encrypt, random_bytes, Key};
use super::header::{associated_data, Header, HEADER_LEN, NONCE_LEN, SALT_LEN};
use super::{KdfParams, VaultError};
use crate::record::{parse_vault_data, VaultData};
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct LegacyKeys {
    /// What the passphrase turned into. An upgrade reuses it to wrap the new vault key.
    pub key: Key,
    pub salt: [u8; SALT_LEN],
    pub params: KdfParams,
}

impl LegacyKeys {
    pub fn verify_passphrase(&self, passphrase: &str) -> Result<(), VaultError> {
        let key = derive_key(passphrase, &self.salt, self.params)?;
        if *key == *self.key { Ok(()) } else { Err(VaultError::Decrypt) }
    }

    pub fn open(bytes: &[u8], passphrase: &str) -> Result<(Self, VaultData), VaultError> {
        let header = Header::decode(bytes)?;
        let key = derive_key(passphrase, &header.salt, header.params)?;
        let keys = Self { key, salt: header.salt, params: header.params };
        let data = keys.unseal(bytes)?;
        Ok((keys, data))
    }

    pub fn seal(&self, data: &VaultData) -> Result<Vec<u8>, VaultError> {
        let header = Header { params: self.params, salt: self.salt, nonce: random_bytes::<NONCE_LEN>()? };
        let encoded_header = header.encode();
        let plaintext = Zeroizing::new(serde_json::to_vec(data)?);
        let ciphertext = encrypt(&self.key, &header.nonce, &plaintext, associated_data(&encoded_header))?;
        Ok([encoded_header.as_slice(), &ciphertext].concat())
    }

    /// Decrypts another copy of this vault with the key already held.
    pub fn unseal(&self, file: &[u8]) -> Result<VaultData, VaultError> {
        let header = Header::decode(file)?;
        let plaintext = decrypt(&self.key, &header.nonce, &file[HEADER_LEN..], associated_data(file))?;
        Ok(parse_vault_data(&plaintext)?)
    }
}
