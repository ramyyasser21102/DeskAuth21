//! Whether the screen is locked right now, read from the operating system.
//!
//! macOS and Windows can answer. Linux desktops have no single answer, so there it
//! always reports "not locked" and only sleep can lock the vault.

pub fn is_screen_locked() -> bool {
    platform::is_locked()
}

#[cfg(target_os = "macos")]
mod platform {
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::string::CFString;

    /// Present, and true, only while the screen is locked.
    const LOCKED_KEY: &str = "CGSSessionScreenIsLocked";

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn CGSessionCopyCurrentDictionary() -> CFDictionaryRef;
    }

    pub fn is_locked() -> bool {
        // SAFETY: the function follows the Create rule, so a non-null result is a new
        // dictionary that this code owns and releases when `session` is dropped.
        let raw = unsafe { CGSessionCopyCurrentDictionary() };
        if raw.is_null() {
            return false;
        }
        let session: CFDictionary<CFString, CFType> = unsafe { CFDictionary::wrap_under_create_rule(raw) };
        session
            .find(CFString::new(LOCKED_KEY))
            .and_then(|value| value.downcast::<CFBoolean>())
            .is_some_and(bool::from)
    }
}

#[cfg(windows)]
mod platform {
    use windows_sys::Win32::System::StationsAndDesktops::{CloseDesktop, OpenInputDesktop, DESKTOP_SWITCHDESKTOP};

    /// While the lock screen (or a permission prompt) owns the input desktop, a normal
    /// program is refused access to it.
    pub fn is_locked() -> bool {
        // SAFETY: plain Win32 calls; a handle that was opened is closed again.
        let desktop = unsafe { OpenInputDesktop(0, 0, DESKTOP_SWITCHDESKTOP) };
        if (desktop as usize) == 0 {
            return true;
        }
        unsafe { CloseDesktop(desktop) };
        false
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    pub fn is_locked() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_the_real_state_does_not_crash() {
        // Whether it is true depends on the machine running the test; it only has to answer.
        let _ = is_screen_locked();
    }
}
