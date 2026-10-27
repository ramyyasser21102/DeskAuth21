export interface WelcomePoint {
  title: string;
  detail: string;
}

export const WELCOME_SUMMARY =
  "A free authenticator for your computer. It shows the same six-digit codes as Google Authenticator and similar apps, right in your menu bar or system tray.";

export const WELCOME_STEPS: readonly WelcomePoint[] = [
  { title: "Bring your accounts over", detail: "Import from Google Authenticator, Aegis, 2FAS, Bitwarden and more, or scan any service's QR code." },
  { title: "Click a code to copy it", detail: "Codes also live in the tray menu, and Cmd/Ctrl+Shift+A opens the window." },
  { title: "Optionally sync", detail: "Keep the vault in a folder iCloud, Dropbox or Syncthing already syncs." },
];

export const SECURITY_POINTS: readonly WelcomePoint[] = [
  { title: "Encrypted on this computer", detail: "Your accounts are sealed with a key made from your passphrase (Argon2id) and XChaCha20-Poly1305." },
  { title: "Your passphrase is never saved", detail: "Only you can open the vault, so a forgotten passphrase cannot be recovered." },
  { title: "No accounts, no servers, no tracking", detail: "DeskAuth21 makes no network calls and works fully offline." },
  { title: "Sync shares only an encrypted file", detail: "Your sync service never sees your accounts." },
  { title: "Locks itself", detail: "After a few idle minutes, and copied codes are cleared from the clipboard after 30 seconds." },
];
