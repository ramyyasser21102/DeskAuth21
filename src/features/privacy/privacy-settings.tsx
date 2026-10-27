import { useState } from "react";
import { CaptureOffConfirmation } from "./capture-off-confirmation";
import { PrivacySwitch } from "./privacy-switch";
import { usePrivacySettings } from "./privacy-settings-provider";
import { usePrivacyActions } from "./use-privacy-actions";

const CAPTURE_DESCRIPTION = "Asks the system to keep this window out of screenshots, recordings and screen sharing.";
const CAPTURE_UNSUPPORTED = "This system has no way to block screen capture, so this cannot be turned on.";
const HIDE_DESCRIPTION = "Show dots instead of digits until you press the eye button. Copying a code never needs it shown.";
const TRAY_DESCRIPTION = "List codes in the menu bar menu. Off shows only account names; clicking one still copies its code.";

export function PrivacySettings() {
  const { settings } = usePrivacySettings();
  const actions = usePrivacyActions();
  const [isConfirmingOff, setIsConfirmingOff] = useState(false);
  if (!settings) return null;

  const handleCapture = (isOn: boolean) => (isOn ? void actions.chooseCaptureProtection(true) : setIsConfirmingOff(true));
  const confirmOff = () => {
    setIsConfirmingOff(false);
    void actions.chooseCaptureProtection(false);
  };

  return (
    <div className="flex flex-col gap-4">
      <span className="text-sm font-medium">Privacy</span>
      <PrivacySwitch id="capture-protection" label="Block screen capture" isOn={settings.capture_protection && settings.capture_protection_supported}
        isDisabled={!settings.capture_protection_supported} onChange={handleCapture}
        description={settings.capture_protection_supported ? CAPTURE_DESCRIPTION : CAPTURE_UNSUPPORTED} />
      {isConfirmingOff && <CaptureOffConfirmation onTurnOff={confirmOff} onKeep={() => setIsConfirmingOff(false)} />}
      <PrivacySwitch id="hide-codes" label="Hide codes until revealed" isOn={settings.hide_codes} onChange={(isOn) => void actions.chooseHiddenCodes(isOn)} description={HIDE_DESCRIPTION} />
      <PrivacySwitch id="tray-codes" label="Show codes in the menu bar menu" isOn={settings.tray_shows_codes} onChange={(isOn) => void actions.chooseTrayCodes(isOn)} description={TRAY_DESCRIPTION} />
      {actions.errorMessage && <p role="alert" className="text-xs text-destructive">{actions.errorMessage}</p>}
    </div>
  );
}
