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

Planned: biometric/keychain unlock, auto-lock, code search, CLI, end-to-end
encrypted sync.

## Development

```sh
npm install
npm run tauri dev   # run the app
npm run check       # eslint, clippy (-D warnings), Rust tests, vitest, tsc
```

## Workflow

Changes land via short-lived feature branches and pull requests into `main`.
