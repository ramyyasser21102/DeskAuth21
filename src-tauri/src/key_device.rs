//! What the vault needs from a physical security key, kept apart from any particular
//! way of talking to one so the vault logic can be tested with a stand-in.

/// Why a key could not be used. The messages tell the person what to do next.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeyError {
    #[error("no hardware key was found; plug it in and try again")]
    NoKey,
    #[error("this is not one of the hardware keys registered for this vault")]
    NotThisKey,
    #[error("the key's PIN was wrong or is needed")]
    PinProblem,
    #[error("the key was not touched in time; try again and touch it when it blinks")]
    NotTouched,
    #[error("hardware keys are not supported on this system yet")]
    #[cfg_attr(not(any(windows, test)), allow(dead_code))]
    Unsupported,
    #[error("the hardware key reported a problem: {0}")]
    Device(String),
}

/// A security key that can make a credential and then derive a secret from it, using the
/// FIDO2 hmac-secret feature. Both steps need the person to touch the key.
pub trait KeyDevice {
    /// Creates a credential on the key and returns its id.
    fn register(&self, pin: Option<&str>) -> Result<Vec<u8>, KeyError>;

    /// The 32 bytes this key derives from `credential_id` and `salt`; the same inputs
    /// always give the same bytes, and only this key can produce them.
    fn derive_secret(&self, credential_id: &[u8], salt: &[u8; 32], pin: Option<&str>) -> Result<[u8; 32], KeyError>;
}

/// The key to use for registering, with the PIN the person typed (if any) and the name they gave it.
pub struct KeyRegistration<'a> {
    pub device: &'a dyn KeyDevice,
    pub pin: Option<&'a str>,
    pub label: &'a str,
}

/// Used where no real key support exists, so asking for one fails with a clear message.
#[cfg(any(windows, test))]
pub struct NoKeySupport;

#[cfg(any(windows, test))]
impl KeyDevice for NoKeySupport {
    fn register(&self, _pin: Option<&str>) -> Result<Vec<u8>, KeyError> {
        Err(KeyError::Unsupported)
    }

    fn derive_secret(&self, _credential_id: &[u8], _salt: &[u8; 32], _pin: Option<&str>) -> Result<[u8; 32], KeyError> {
        Err(KeyError::Unsupported)
    }
}
