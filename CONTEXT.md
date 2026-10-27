# DeskAuth21

A desktop app that holds a person's two-factor accounts and shows their one-time codes, so a phone is not needed. Its promise is privacy: nothing leaves the person's computer except an encrypted file they choose to sync.

## Language

**Vault**:
The single encrypted file that holds every account. It is what gets synced between computers.
_Avoid_: database, store, backup

**Account**:
One service the person signs in to with two-factor codes, such as GitHub or a bank.
_Avoid_: entry, login, key

**Seed**:
The long-lived secret kept for an account, from which its codes are calculated. Anyone who has a seed can produce that account's codes forever.
_Avoid_: key, secret, token

**Code**:
The short number that changes every 30 or 60 seconds and is typed into a service to sign in.
_Avoid_: OTP, token, key

**Source app**:
The authenticator app an account was brought over from, kept as a label on the account.
_Avoid_: vendor, provider

**Passphrase**:
What the person types to unlock the vault. It is never stored anywhere and cannot be recovered if forgotten.
_Avoid_: password, master password, PIN

**Lock** / **Unlock**:
Locking removes the seeds from the app's memory so the vault must be unlocked with the passphrase again.
_Avoid_: log out, sign out

**Auto-lock**:
The vault locking itself without being asked, for example after idle time, when the computer sleeps, or when the screen locks.

**Capture protection**:
The window asking the system not to include it in screenshots or screen recordings. It is a request the system may not honour, not a guarantee.
_Avoid_: screenshot blocking, DRM

**Revealed code**:
A code shown as digits. By default codes are hidden and shown only briefly after the person asks. Copying a code never requires revealing it.
_Avoid_: unmasked, visible code

**Tray menu**:
The menu opened from the menu bar or system tray icon, where codes can be copied without opening the window.
_Avoid_: dropdown, status menu

**Second factor**:
An extra thing besides the passphrase, such as a hardware key or a phone, that is needed to unlock the vault. A second factor that takes part in encrypting the vault is a **key factor**; one that only approves unlocking is a **gate factor**.
_Avoid_: 2FA (this app's whole subject is 2FA codes, so the word is ambiguous here)

**Hardware key**:
A small physical security key (USB or NFC) that the person can register as a key factor, so the vault cannot be opened without it. It works with no network.
_Avoid_: token, YubiKey (a brand), passkey

**Recovery key**:
A long random code created when a second factor is added, shown once for the person to store safely. It is the only way back in if the passphrase or every hardware key is lost.
_Avoid_: backup code, reset code

**Biometric unlock**:
Opening the vault on one particular computer with a fingerprint or face instead of typing, after the vault has been fully unlocked there once. It belongs to that computer and never makes the vault openable anywhere else.
_Avoid_: passkey, Touch ID unlock (a brand name for one platform)

