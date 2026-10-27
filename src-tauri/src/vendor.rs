//! The app an account came from. A tag for organising and switching between
//! accounts; it does not change how codes are generated.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vendor {
    GoogleAuthenticator,
    MicrosoftAuthenticator,
    Authy,
    OnePassword,
    Bitwarden,
    LastPass,
    DuoMobile,
    OktaVerify,
    Aegis,
    TwoFas,
    ProtonAuthenticator,
    EnteAuth,
    FreeOtp,
    AndOtp,
    Raivo,
    /// Anything else, and accounts saved before vendors existed.
    #[default]
    Other,
}

/// In the order shown to the user: the most widely used first, "Other" last.
pub const ALL_VENDORS: [Vendor; 16] = [
    Vendor::GoogleAuthenticator,
    Vendor::MicrosoftAuthenticator,
    Vendor::Authy,
    Vendor::OnePassword,
    Vendor::Bitwarden,
    Vendor::LastPass,
    Vendor::DuoMobile,
    Vendor::OktaVerify,
    Vendor::Aegis,
    Vendor::TwoFas,
    Vendor::ProtonAuthenticator,
    Vendor::EnteAuth,
    Vendor::FreeOtp,
    Vendor::AndOtp,
    Vendor::Raivo,
    Vendor::Other,
];

impl Vendor {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::GoogleAuthenticator => "Google Authenticator",
            Self::MicrosoftAuthenticator => "Microsoft Authenticator",
            Self::Authy => "Authy",
            Self::OnePassword => "1Password",
            Self::Bitwarden => "Bitwarden",
            Self::LastPass => "LastPass Authenticator",
            Self::DuoMobile => "Duo Mobile",
            Self::OktaVerify => "Okta Verify",
            Self::Aegis => "Aegis Authenticator",
            Self::TwoFas => "2FAS",
            Self::ProtonAuthenticator => "Proton Authenticator",
            Self::EnteAuth => "Ente Auth",
            Self::FreeOtp => "FreeOTP",
            Self::AndOtp => "andOTP",
            Self::Raivo => "Raivo OTP",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VendorInfo {
    pub id: Vendor,
    pub name: &'static str,
}

pub fn vendor_catalog() -> Vec<VendorInfo> {
    ALL_VENDORS.iter().map(|&id| VendorInfo { id, name: id.display_name() }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_vendor_is_listed_exactly_once_with_other_last() {
        let unique: HashSet<_> = ALL_VENDORS.iter().collect();
        assert_eq!(unique.len(), ALL_VENDORS.len());
        assert_eq!(ALL_VENDORS.last(), Some(&Vendor::Other));
    }

    #[test]
    fn names_are_distinct_and_not_empty() {
        let names: HashSet<_> = ALL_VENDORS.iter().map(|vendor| vendor.display_name()).collect();
        assert_eq!(names.len(), ALL_VENDORS.len());
        assert!(names.iter().all(|name| !name.is_empty()));
    }

    #[test]
    fn ids_are_stable_snake_case_strings_the_frontend_can_rely_on() {
        assert_eq!(serde_json::to_string(&Vendor::GoogleAuthenticator).unwrap(), "\"google_authenticator\"");
        assert_eq!(serde_json::to_string(&Vendor::TwoFas).unwrap(), "\"two_fas\"");
        assert_eq!(serde_json::from_str::<Vendor>("\"one_password\"").unwrap(), Vendor::OnePassword);
    }

    #[test]
    fn the_default_is_other() {
        assert_eq!(Vendor::default(), Vendor::Other);
    }
}
