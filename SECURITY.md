# Security policy

DeskAuth21 keeps your one-time-code seeds, so a flaw in it matters. Thank you for
reporting one responsibly.

## Reporting a vulnerability

Please **do not open a public issue** for a security problem. Use GitHub's private
reporting instead:

<https://github.com/ramyyasser21102/DeskAuth21/security/advisories/new>

Include what you found, the steps to reproduce it, the version and operating system, and
what an attacker could gain. You do not need a fix or a proof of concept.

You can expect an acknowledgement within a week. This is a volunteer project, so fixes take
the time they take, but a confirmed vulnerability comes before feature work. You will be told
when it is fixed and credited in the release notes unless you prefer not to be.

## Supported versions

Only the latest release and the `main` branch receive security fixes. There are no
long-term support branches.

## Scope

In scope:

- The vault file: its encryption, key wrapping, unlock slots, recovery key and hardware key.
- Leaks of seeds, codes, the vault key or the passphrase to logs, disk, the clipboard past its
  clearing time, or the network.
- Locking, auto-lock, lock on sleep, screen-capture protection and hidden codes failing to do
  what the README says.
- Importers handling hostile files (crashes, path or memory problems).
- Sync: another device or a synced folder being able to read or silently overwrite the vault.

Out of scope, and documented in the README under "Privacy and its limits":

- Malware or an attacker already running as you on an unlocked computer. No authenticator can
  protect against that.
- Other screen recorders ignoring the request not to capture the window, and clipboard
  managers keeping their own copy.
- Builds not being code-signed, and the first-run warnings that follow.
- Someone who has your passphrase (and your hardware key, if one is required).

## How the app protects you

The design, the format and the reasoning are written down:

- [`docs/security-and-privacy-plan.md`](docs/security-and-privacy-plan.md)
- [`docs/vault-format.md`](docs/vault-format.md)
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)

The app makes no network requests and sends no telemetry. The vault is encrypted on disk, and
a synced folder only ever sees that encrypted file.

## Please

Test only on your own vault and your own devices, and give us reasonable time to fix a problem
before you share it.
