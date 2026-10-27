//! A real security key over USB, using the FIDO2 hmac-secret feature.
//!
//! Windows does not let ordinary programs open these devices directly (it wants the
//! WebAuthn API), so there keys are reported as unsupported until that is built.

use crate::key_device::{KeyDevice, KeyError};
#[cfg(windows)]
use crate::key_device::NoKeySupport;

pub const fn is_supported() -> bool {
    cfg!(not(windows))
}

/// The key device for this system.
pub fn platform_device() -> Box<dyn KeyDevice> {
    #[cfg(not(windows))]
    return Box::new(real::UsbKey);
    #[cfg(windows)]
    return Box::new(NoKeySupport);
}

/// Sorts the library's error text into what the person can do about it. The library reports
/// device errors as text containing the CTAP status name.
pub fn classify_error(message: &str) -> KeyError {
    let lowered = message.to_lowercase();
    if lowered.contains("no_credentials") {
        KeyError::NotThisKey
    } else if lowered.contains("pin_invalid") || lowered.contains("pin_required") || lowered.contains("pin_auth") || lowered.contains("pin_not_set") {
        KeyError::PinProblem
    } else if lowered.contains("action_timeout") || lowered.contains("user_action") || lowered.contains("operation_denied") || lowered.contains("timeout") {
        KeyError::NotTouched
    } else if lowered.contains("device not found") || lowered.contains("no device") || lowered.contains("hid") && lowered.contains("not found") {
        KeyError::NoKey
    } else {
        KeyError::Device(message.to_string())
    }
}

#[cfg(not(windows))]
mod real {
    use super::classify_error;
    use crate::key_device::{KeyDevice, KeyError};
    use ctap_hid_fido2::fidokey::get_assertion::get_assertion_params::Extension as Gext;
    use ctap_hid_fido2::fidokey::make_credential::Mext;
    use ctap_hid_fido2::fidokey::{GetAssertionArgsBuilder, MakeCredentialArgsBuilder};
    use ctap_hid_fido2::{Cfg, FidoKeyHid, FidoKeyHidFactory};

    /// Not a website: the key only uses this name to tell credentials apart.
    const RELYING_PARTY: &str = "deskauth21.local";
    /// Nothing verifies the signature, so the challenge does not need to be secret or fresh.
    const CHALLENGE: [u8; 32] = *b"DeskAuth21 hardware key challeng";

    pub struct UsbKey;

    fn open() -> Result<FidoKeyHid, KeyError> {
        let quiet = Cfg { keep_alive_msg: String::new(), ..Cfg::init() };
        FidoKeyHidFactory::create(&quiet).map_err(|error| match classify_error(&error.to_string()) {
            KeyError::Device(_) => KeyError::NoKey,
            other => other,
        })
    }

    impl KeyDevice for UsbKey {
        fn register(&self, pin: Option<&str>) -> Result<Vec<u8>, KeyError> {
            let builder = MakeCredentialArgsBuilder::new(RELYING_PARTY, &CHALLENGE).extensions(&[Mext::HmacSecret(Some(true))]);
            let args = match pin {
                Some(pin) => builder.pin(pin),
                None => builder.without_pin_and_uv(),
            }
            .build();
            let attestation = open()?.make_credential_with_args(&args).map_err(|error| classify_error(&error.to_string()))?;
            Ok(attestation.credential_descriptor.id)
        }

        fn derive_secret(&self, credential_id: &[u8], salt: &[u8; 32], pin: Option<&str>) -> Result<[u8; 32], KeyError> {
            let builder = GetAssertionArgsBuilder::new(RELYING_PARTY, &CHALLENGE).credential_id(credential_id).extensions(&[Gext::HmacSecret(Some(*salt))]);
            let args = match pin {
                Some(pin) => builder.pin(pin),
                None => builder.without_pin_and_uv(),
            }
            .build();
            let assertions = open()?.get_assertion_with_args(&args).map_err(|error| classify_error(&error.to_string()))?;
            secret_from(&assertions).ok_or_else(|| KeyError::Device("the key did not return a secret".to_string()))
        }
    }

    fn secret_from(assertions: &[ctap_hid_fido2::fidokey::get_assertion::get_assertion_params::Assertion]) -> Option<[u8; 32]> {
        let mut extensions = assertions.iter().flat_map(|assertion| assertion.extensions.iter());
        extensions.find_map(secret_in)
    }

    fn secret_in(extension: &Gext) -> Option<[u8; 32]> {
        match extension {
            Gext::HmacSecret(output) => *output,
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
