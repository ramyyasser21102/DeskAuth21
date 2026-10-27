# Security and privacy plan

Agreed in a design discussion; terms are defined in `CONTEXT.md`, the second-factor decision is in `docs/adr/0001-hardware-key-second-factor-with-recovery-key.md`.

## What we defend against

A copied vault file, a screenshot or recording of the screen, someone looking over a shoulder, and someone at an unlocked computer. For malware already running on the computer we apply best-effort mitigations and promise nothing; the README and the welcome page say so. Code signing is not planned, so other programs on the computer can attach to the app and macOS shows its first-launch warning; this is stated as a limit.

## Decisions

| Area | Decision |
|---|---|
| Capture protection | On by default, with a Settings switch. A permanent banner shows whenever it is off or the system cannot provide it, and turning it off asks for confirmation. |
| Hidden codes | Digits are hidden by default (a setting). One eye button per row reveals that code for 10 seconds; it hides again if the window loses focus or the app locks. Clicking a row copies without revealing. |
| Tray menu | A setting. By default it lists labels only; clicking a row copies the code. |
| Auto-lock | Also lock on sleep and on screen lock (a setting). Closing the window to the tray does not lock. |
| Clipboard | 15 seconds by default (a setting), and cleared when the vault locks or the app quits. |
| Passphrase | New vaults: at least 12 characters, a strength meter, a generator (six random words), and slower key derivation (128 MiB). Existing vaults keep their settings until the passphrase changes. |
| Second factor | Optional hardware key as a key factor; passphrase and key are both required. Several keys allowed; a warning when only one is registered. Mandatory 24-word recovery key. No network. See the ADR. |
| Biometric unlock | Touch ID or Windows Hello on one computer. Full credentials are required again after an app restart, after 14 days, and when enrolled fingerprints change. Not offered where the system refuses it. |
| Existing vaults | Upgrading to the new format is asked once, with a safety copy named so it is not merged as a vault. |
| Phone passkey | Parked: needs a network relay and signing. |

## Build order (one pull request each)

1. Capture protection, the banner and confirmation, hidden codes with the eye button, tray labels-only mode.
2. Lock on sleep and screen lock; the clipboard changes.
3. Passphrase rules, strength meter, generator, stronger key derivation.
4. New vault format with unlock slots, change passphrase, recovery key, upgrade flow.
5. Hardware key support.
6. Biometric unlock.

Steps 1 to 3 need no format change.

## To verify while building

- Capture protection: tested only with the macOS screenshot tool. Third-party recorders (OBS, Zoom) and Windows are untested; Linux has no equivalent.
- Hardware keys: which library to use and whether it works on macOS, Windows and Linux (Linux needs device permissions).
- Biometric unlock: whether an unsigned macOS build can use Touch ID and the secure storage; Windows Hello via a community plugin is unverified.
