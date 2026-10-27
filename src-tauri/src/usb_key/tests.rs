use super::*;

#[test]
fn windows_is_the_one_system_reported_as_unsupported() {
    assert_eq!(is_supported(), cfg!(not(windows)));
}

#[cfg(not(windows))]
mod classification {
    use super::super::*;


    #[test]
    fn a_credential_the_key_does_not_hold_means_it_is_not_this_key() {
        assert_eq!(classify_error("0x2E CTAP2_ERR_NO_CREDENTIALS    No valid credentials provided."), KeyError::NotThisKey);
    }

    #[test]
    fn pin_problems_are_reported_as_pin_problems() {
        for text in ["0x31 CTAP2_ERR_PIN_INVALID   PIN Invalid.", "0x36 CTAP2_ERR_PIN_REQUIRED  PIN is required", "0x33 CTAP2_ERR_PIN_AUTH_INVALID"] {
            assert_eq!(classify_error(text), KeyError::PinProblem, "{text}");
        }
    }

    #[test]
    fn a_missing_touch_is_reported_as_such() {
        assert_eq!(classify_error("0x27 CTAP2_ERR_OPERATION_DENIED"), KeyError::NotTouched);
        assert_eq!(classify_error("0x3A CTAP2_ERR_ACTION_TIMEOUT"), KeyError::NotTouched);
    }

    #[test]
    fn anything_else_keeps_the_original_text_for_the_person_to_read() {
        assert_eq!(classify_error("something odd"), KeyError::Device("something odd".to_string()));
    }
}
