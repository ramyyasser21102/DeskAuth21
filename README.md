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
- Sync between computers by keeping the vault in a synced folder (see below)

Planned: biometric/keychain unlock, lock on sleep/screen lock, code search, CLI, deleting and
renaming accounts (and syncing those), a self-hosted relay for faster sync.

## Syncing between devices

DeskAuth21 does not run a server. To use the same accounts on several computers, keep the
vault in a folder that iCloud Drive, Dropbox, OneDrive, Syncthing or similar already
syncs. Only the encrypted file is shared, so the sync service never sees your accounts.

1. On the first computer, open **Vault location** and **Choose folder...**, picking a
   synced folder. Your vault is moved there.
2. On each other computer, install DeskAuth21 and choose **Already have a vault in a
   synced folder?**, pick the same folder, then unlock with the **same passphrase**.

While unlocked, the app checks the file every second. Accounts added on one computer
show up on the others as soon as the sync service delivers the file, and the app merges
instead of overwriting, so two computers adding accounts at the same time keep both.
If the file is replaced by a different vault, the app refuses to save over it.

Limits for now: accounts can only be added (not deleted or renamed), so there is nothing
else to sync; "conflicted copy" files made by a sync service are not merged; and
speed depends on how quickly your sync service delivers the file.

## Development

```sh
npm install
npm run tauri dev   # run the app
npm run check       # eslint, clippy (-D warnings), Rust tests, vitest, tsc
```

## Workflow

Changes land via short-lived feature branches and pull requests into `main`.
