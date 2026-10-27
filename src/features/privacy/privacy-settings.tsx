import { CaptureProtectionSetting } from "./capture-protection-setting";
import { ClipboardClearSetting } from "./clipboard-clear-setting";
import { PrivacySwitch } from "./privacy-switch";
import { usePrivacySettings } from "./privacy-settings-provider";
import { usePrivacyActions } from "./use-privacy-actions";

const HIDE_DESCRIPTION = "Show dots instead of digits until you press the eye button. Copying a code never needs it shown.";
const TRAY_DESCRIPTION = "List codes in the tray menu. Off shows only account names; clicking one still copies its code.";
const LOCK_DESCRIPTION = "Locks the vault as soon as the computer wakes from sleep. The screen lock is noticed on macOS and Windows; on Linux only sleep is.";

export function PrivacySettings() {
  const { settings } = usePrivacySettings();
  const actions = usePrivacyActions();
  if (!settings) return null;

  return (
    <div className="flex flex-col gap-4">
      <span className="text-sm font-medium">Privacy</span>
      <CaptureProtectionSetting settings={settings} onChoose={(enabled) => void actions.chooseCaptureProtection(enabled)} />
      <PrivacySwitch id="hide-codes" label="Hide codes until revealed" isOn={settings.hide_codes} onChange={(isOn) => void actions.chooseHiddenCodes(isOn)} description={HIDE_DESCRIPTION} />
      {settings.tray_menu_lists_accounts && (
        <PrivacySwitch id="tray-codes" label="Show codes in the tray menu" isOn={settings.tray_shows_codes} onChange={(isOn) => void actions.chooseTrayCodes(isOn)} description={TRAY_DESCRIPTION} />
      )}
      <PrivacySwitch id="lock-on-sleep" label="Lock when the computer sleeps or the screen locks" isOn={settings.lock_on_sleep} onChange={(isOn) => void actions.chooseLockOnSleep(isOn)} description={LOCK_DESCRIPTION} />
      <ClipboardClearSetting seconds={settings.clipboard_clear_seconds} onChoose={(seconds) => void actions.chooseClipboardSeconds(seconds)} />
      {actions.errorMessage && <p role="alert" className="text-xs text-destructive">{actions.errorMessage}</p>}
    </div>
  );
}
