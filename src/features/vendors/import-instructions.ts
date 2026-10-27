import type { VendorId } from "@/lib/api";

const NO_EXPORT =
  "has no export of its codes. For each service, scan or paste the QR code or otpauth:// link shown when you set up two-factor sign-in (or when you add a new device).";
const PER_ACCOUNT = "Choose a QR image or paste an otpauth:// link for each account.";
const OTPAUTH_LINES = "Export your accounts as otpauth:// links (a text file) and choose the file or paste the links.";

/** How to get accounts out of each app, in the words its own menus use. */
export const IMPORT_INSTRUCTIONS: Readonly<Record<VendorId, string>> = {
  google_authenticator:
    "In Google Authenticator choose Transfer accounts, then Export accounts. Choose the QR images (or paste a screenshot). Large exports span several QR codes; choose them all.",
  microsoft_authenticator: `Microsoft Authenticator ${NO_EXPORT}`,
  authy: `Authy ${NO_EXPORT}`,
  one_password: "1Password stores a code per login. Open the item, copy the one-time password's otpauth:// link or QR code, and import it here.",
  bitwarden: "Export your vault as unencrypted .json or .csv and choose the file. Steam codes are skipped.",
  last_pass: `LastPass Authenticator ${NO_EXPORT}`,
  duo_mobile: `Duo Mobile ${NO_EXPORT}`,
  okta_verify: `Okta Verify ${NO_EXPORT}`,
  aegis: "Export the vault without encryption (Settings, Export) and choose the .json file.",
  two_fas: "Create a backup without a password in 2FAS and choose the file.",
  proton_authenticator: PER_ACCOUNT,
  ente_auth: OTPAUTH_LINES,
  free_otp: PER_ACCOUNT,
  and_otp: PER_ACCOUNT,
  raivo: PER_ACCOUNT,
  other: "Choose QR images or an export file, or paste an otpauth:// link. Accounts will be tagged Other.",
};
