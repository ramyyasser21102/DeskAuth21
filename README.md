# DeskAuth21

A free, private, cross-platform desktop authenticator (TOTP) that lives in your
menu bar (macOS/Linux) or system tray (Windows). Built with Tauri 2 (Rust + TypeScript).

## Status

Early development. Currently working:

- TOTP generation (RFC 6238; SHA1/SHA256/SHA512, 6 or 8 digits)
- Passphrase-encrypted vault (Argon2id + XChaCha20-Poly1305) with lock/unlock
- Import from Google Authenticator exports: QR images, pasted screenshots or links
- Tray menu with live codes; click a code to copy it (cleared after 30 seconds)
- Closing the window keeps the app running in the tray
- Global hotkey `Cmd/Ctrl+Shift+A` shows or hides the window
- "Launch at login" toggle in the tray menu (starts hidden in the tray)
- Auto-lock after 1, 5, 15 or 30 minutes without activity, or never (default 5)
- Rename and delete accounts (a delete asks for confirmation and warns about losing access)
- Sync between computers by keeping the vault in a synced folder (see below)

Planned: biometric/keychain unlock, lock on sleep/screen lock, code search, CLI, changing the
passphrase, a self-hosted relay for faster sync.

## Importing accounts

Each account is tagged with the app it came from (Google Authenticator, Microsoft
Authenticator, Authy, 1Password, Bitwarden, LastPass, Duo Mobile, Okta Verify, Aegis, 2FAS,
Proton Authenticator, Ente Auth, FreeOTP, andOTP, Raivo OTP, or Other).

| Source | How to import | Notes |
|---|---|---|
| Google Authenticator | Transfer accounts, then Export; choose the QR images (or paste a screenshot) | Large exports span several QR codes; choose them all |
| Aegis | Export the vault **without encryption** and choose the `.json` file | |
| 2FAS | Export a backup **without a password** and choose the file | |
| Bitwarden | Export as unencrypted `.json` or `.csv` and choose the file | Steam codes are skipped |
| Ente Auth, others that export `otpauth://` links | Choose the text file, or paste the links | One link per line |
| Microsoft Authenticator, Authy, 1Password, LastPass, Duo, Okta, ... | Scan or paste each service's own QR code or `otpauth://` link | These apps offer no export of their codes; pick the app so the account is tagged |

Password-protected exports are not supported yet; the app says so and asks for an
unprotected one. Counter-based (HOTP) and Steam entries are skipped and counted, so one
odd entry never costs you the rest of a file. The same account imported from two apps is
recognised by its secret and added once.

## Syncing between devices

DeskAuth21 does not run a server. To use the same accounts on several computers, keep the
vault in a folder that iCloud Drive, Dropbox, OneDrive, Syncthing or similar already
syncs. Only the encrypted file is shared, so the sync service never sees your accounts.

1. On the first computer, open **Vault location** and **Choose folder...**, picking a
   synced folder. Your vault is moved there.
2. On each other computer, install DeskAuth21 and choose **Open a vault from a
   synced folder**, pick the same folder, then unlock with the **same passphrase**.

While unlocked, the app checks the file every second. Accounts added on one computer
show up on the others as soon as the sync service delivers the file, and the app merges
instead of overwriting, so two computers adding accounts at the same time keep both.
If the file is replaced by a different vault, the app refuses to save over it.

Renames and deletes sync too: for each account the newest change wins, a delete is
remembered so a computer that still has the account cannot bring it back, and
re-importing a deleted account on purpose adds it again. Computers' clocks are only used
to order changes, and a new change is always treated as newer than anything the computer
has already seen, so a slightly wrong clock cannot make an edit lose.

If two computers save at nearly the same moment, sync services leave a second file beside
the vault ("conflicted copy", "sync-conflict", "vault 2" and similar). DeskAuth21 merges any
such file that opens with your passphrase's key, then renames it to end in `.merged` instead
of deleting it, so you can always inspect or undo a merge by hand. Files from a different
vault are never touched.

Limits for now: speed depends on how quickly your sync service delivers the file, and all
computers must use the same passphrase.

### Trying sync on one computer

Build the app once, then run two copies with different data folders. (Two `tauri dev` runs
would fight over the dev server port, so use the built binary.)

```sh
npm run tauri build -- --no-bundle
DESKAUTH21_DATA_DIR=/tmp/deskauth-a src-tauri/target/release/deskauth21 &
DESKAUTH21_DATA_DIR=/tmp/deskauth-b src-tauri/target/release/deskauth21 &
```

In each, create a vault with the same passphrase, open Settings and choose the same folder as
the vault location (the second can use "Open a vault from a synced folder" on its first
screen). Add, rename or delete an account in one and watch it appear in the other. Each copy
has its own tray icon, so tell them apart by their windows.

## Installing a build

Installers for macOS, Windows and Linux are attached to the
[Releases](https://github.com/ramyyasser21102/DeskAuth21/releases) page. They are **unsigned**
(there are no Apple or Microsoft developer certificates yet), so each system warns the
first time:

- **macOS**: open the `.dmg` and drag DeskAuth21 to Applications. The first time, right-click
  the app and choose **Open**, or run `xattr -dr com.apple.quarantine /Applications/DeskAuth21.app`.
- **Windows**: SmartScreen shows "Windows protected your PC". Choose **More info**, then
  **Run anyway**.
- **Linux**: make the `.AppImage` executable (`chmod +x`) and run it, or install the `.deb`.
  The tray icon needs AppIndicator support; GNOME needs the "AppIndicator and
  KStatusNotifierItem Support" extension. Global hotkeys generally do not work under Wayland,
  so use the tray menu there.

To build it yourself: `npm ci && npm run tauri build`; the installers land in
`src-tauri/target/release/bundle/`.

### Publishing a release (maintainers)

Pushing a version tag builds all three installers in GitHub Actions and attaches them to a
**draft** release, so nothing is public until you publish it:

```sh
git tag v0.1.0
git push origin v0.1.0
```

Every pull request is also checked on macOS, Windows and Linux (lint, clippy, tests).

## Development

```sh
npm install
npm run tauri dev   # run the app
npm run check       # eslint, clippy (-D warnings), Rust tests, vitest, tsc
```

## Workflow

Changes land via short-lived feature branches and pull requests into `main`.
