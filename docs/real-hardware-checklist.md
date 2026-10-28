# Real-hardware checklist

Automated tests cover the logic, but these things only a person at a real computer can check:
system dialogs, the tray, global shortcuts, hardware keys, screen capture and sleep. Run this once
per operating system before a release and write down anything that differs.

**Use a throwaway vault.** Set `DESKAUTH21_DATA_DIR` to an empty folder (for example
`DESKAUTH21_DATA_DIR=/tmp/da21-test npm run tauri dev`, or the same variable before opening a
built app) so none of this touches your real vault. Import a few test accounts first; the
`otpauth://` links from any service's test setup, or these made-up ones, work:

```
otpauth://totp/Acme:one@example.com?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Acme
otpauth://totp/Globex:two@example.com?secret=MFRGGZDFMZTWQ2LK&issuer=Globex
```

Mark each line **pass / fail / n/a** and note the OS and version at the top.

## 1. Start-up and the window

- [ ] First launch shows the welcome page, animated in, then creating a vault works.
- [ ] The window, tray menu and unlock screen all say "DA21".
- [ ] Closing the window leaves the app in the tray or menu bar; it does not quit or lock.
- [ ] Reopening: **macOS/Windows** a left click on the icon shows the window; **Linux** the icon
      opens the menu and "Open DA21" shows the window.

## 2. Tray and menu bar

- [ ] Right-click (Linux: click) shows: Open DA21, Settings, Lock now, Launch at login, Quit.
- [ ] "Settings" opens the Settings page; "Lock now" is greyed out while locked.
- [ ] "Lock now" locks the vault and the window shows the unlock screen.
- [ ] **Linux only:** the menu lists every account grouped by app, full names, no "…more" item,
      and clicking one copies its code. Codes are hidden unless "Show codes in the tray menu" is on.
- [ ] "Launch at login" toggles, and after a restart or re-login the app starts hidden in the tray.
- [ ] Light and dark system themes both give a visible icon.

## 3. Copying codes

- [ ] Clicking a code copies it: the countdown ring swaps to a check and "Copied", then back.
- [ ] Paste into a text field: it is the right six digits for that account.
- [ ] After the clearing time (default 15 s) pasting gives something else or nothing.
- [ ] Copy, then lock: the clipboard no longer holds the code. Copy, then quit: same.
- [ ] If the clipboard changed to something else in between, locking does not wipe that.

## 4. Keyboard shortcuts

- [ ] Default `Cmd/Ctrl+Shift+A` shows the window from another app, and hides it when in front.
- [ ] Settings > Keyboard shortcuts: set "Lock the vault" to a new combination; pressing it from
      another app locks the vault.
- [ ] A bare key (no modifier) is refused with a message. Reusing one action's combination for
      the other is refused.
- [ ] A combination another app already owns shows the "another app may be using it" error and the
      old shortcut keeps working.
- [ ] Clear removes the shortcut and pressing it does nothing. After a restart the choices persist.
- [ ] **Linux/Wayland:** note whether shortcuts work at all (expected: often not).

## 5. Privacy and locking

- [ ] Codes show as dots; the eye button reveals one for about 10 seconds; losing focus hides it.
- [ ] **macOS/Windows:** take a screenshot and start a screen recording or share: the window does
      not appear (it should be black or missing). Turning "Block screen capture" off asks first and
      shows the banner. **Linux:** the banner says protection is unavailable.
- [ ] Auto-lock: set 1 minute, leave the app idle, it locks. Moving the mouse in the window resets it.
- [ ] Sleep the computer and wake it: the vault is locked. **macOS/Windows:** lock the screen and
      return: locked. **Linux:** only sleep is expected to lock.
- [ ] A background refresh does not count as activity (an idle window still locks on time).

## 6. Hardware key (macOS and Linux; Windows should say unsupported)

Needs a FIDO2 key that supports hmac-secret (YubiKey 5 and similar). Do this on the throwaway vault.

- [ ] Settings > Security shows the hardware-key section. **Windows:** it says keys are unsupported.
- [ ] Create a recovery key first (it is required), write it down, and finish the wizard.
- [ ] "Require a hardware key": enter the passphrase, touch the key when it blinks. It succeeds.
- [ ] Lock, then unlock: passphrase plus a touch opens it. The passphrase alone does not.
- [ ] Unplugging the key gives a clear message, not a hang or a crash.
- [ ] If your key has a PIN: the PIN field works and a wrong PIN is reported.
- [ ] Add a second key (if you have one): either key opens the vault. Remove one: it no longer does.
- [ ] Unlock with the recovery key: it works and the key requirement is dropped; set it up again.
- [ ] "Stop requiring a key" (passphrase asked) returns to passphrase-only unlock.
- [ ] **Linux:** if the key cannot be opened, the error mentions permissions (udev rules).

## 7. Backup, restore and export

- [ ] Settings > Save a backup: the save dialog suggests `DA21-backup-YYYY-MM-DD.backup`; the file
      exists afterwards and the message shows where.
- [ ] Add one more account, then Restore a backup: confirmation starts on Cancel; choosing the
      file locks the vault, the unlock uses the backup's passphrase, and the new account is gone.
      A `*.before-restore.backup` file sits next to the vault.
- [ ] Restoring a file that is not a vault (a text file) is refused and nothing changes.
- [ ] On a fresh data folder, the create-vault screen's "Restore from a backup file" works.
- [ ] Export accounts: the warning shows, a wrong passphrase is refused before any file is written,
      the right one writes a file of `otpauth://` links. Import that file into a new vault: the same
      accounts and codes come back. **Delete the exported file afterwards.**

## 8. Sync between folders

You only need one computer: use two data folders pointing at the same synced or shared folder.

- [ ] Settings > Vault location > Choose folder moves the vault there.
- [ ] A second instance (different `DESKAUTH21_DATA_DIR`, same folder) opens the same vault.
- [ ] Rename an account in one; within a few seconds the other shows it.
- [ ] Editing in both before they sync keeps both changes (a conflict copy merges, nothing is lost).

## 9. Look and feel

- [ ] Page changes fade; the tab line slides; search results glide; the welcome page appears in sequence.
- [ ] With the system's "reduce motion" on, there is no sliding and the countdown ring does not sweep.
- [ ] Both themes are strictly black and white with Montserrat; nothing loads from the network
      (try with the network off: everything still works).
- [ ] The window is usable at its smallest size and at a large size; long account names wrap, not truncate.

## 10. Installing a build

- [ ] The installer or bundle opens. **macOS:** right-click > Open (unsigned warning), and the app
      is named `DA21.app`. **Windows:** SmartScreen "More info > Run anyway". **Linux:** the package
      installs and the app starts.
- [ ] Launch it a second time while it is running and note what happens. There is no
      single-instance guard today, so a second copy opening is expected, not a failure.
- [ ] Quitting from the tray menu really exits, and the clipboard code (if any) is cleared.

## Reporting what fails

For each failure, note the OS and version, whether it is a debug or release build, the exact
message shown, and the steps. Never paste a real secret key into a report; use the test accounts.
