# The second factor is an optional hardware key that joins the vault key, backed by a recovery key

To protect a copied vault file beyond what a passphrase can, the vault can require a hardware key (offline, USB or NFC) in addition to the passphrase, and the key takes part in encrypting the vault rather than only approving an unlock. A phone passkey was wanted first but rejected: its phone-to-computer flow needs an internet relay, which breaks "nothing leaves the machine", and on macOS it needs a signed app plus a domain hosting a verification file. Because losing every key and the passphrase would make the vault permanently unreadable, a 24-word recovery key is mandatory when a hardware key is added, and several keys may be registered.

## Considered Options

- **Phone passkey (WebAuthn over the phone QR flow):** rejected for the network relay, the signing and domain requirement, and passkeys that usually sync through a cloud account.
- **Gate factor only (approves unlocking, file still opens with the passphrase):** rejected because it does nothing against a stolen vault file.

## Consequences

- The vault file format changes: the vault's real key is wrapped by several unlock slots (passphrase plus each hardware key, and the recovery key). Computers still on an older version will refuse the file until they update, so upgrading an existing vault is an explicit, one-time choice with a safety copy.
- Biometric unlock is a deliberate exception on one computer: it keeps the vault key in that computer's secure storage, and full credentials are required again after a restart, after 14 days, or if enrolled fingerprints change.
- With no passphrase, no key and no recovery key, nothing can open the vault. That is intended.
