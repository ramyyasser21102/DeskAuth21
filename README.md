# DeskAuth21

A free, private, cross-platform desktop authenticator (TOTP) that lives in your
menu bar (macOS/Linux) or system tray (Windows). Built with Tauri 2 (Rust + TypeScript).

## Status

Early development. Currently working:

- TOTP generation (RFC 6238; SHA1/SHA256/SHA512, 6 or 8 digits)
- Import from `otpauth://` URIs and Google Authenticator `otpauth-migration://` exports
- Tray menu with live codes (in-memory only)

Planned: encrypted vault, clipboard copy with auto-clear, import UI with QR scanning,
end-to-end encrypted sync, autostart, global hotkey, CLI.

## Development

```sh
npm install
npm run tauri dev      # run the app
cargo test --manifest-path src-tauri/Cargo.toml   # Rust unit tests
```

## Workflow

Changes land via short-lived feature branches and pull requests into `main`.
