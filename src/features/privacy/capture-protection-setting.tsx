import { useState } from "react";
import type { Settings } from "@/lib/api";
import { CaptureOffConfirmation } from "./capture-off-confirmation";
import { PrivacySwitch } from "./privacy-switch";

const DESCRIPTION = "Asks the system to keep this window out of screenshots, recordings and screen sharing.";
const UNSUPPORTED = "This system has no way to block screen capture, so this cannot be turned on.";

interface CaptureProtectionSettingProps {
  settings: Settings;
  onChoose: (enabled: boolean) => void;
}

/** Turning protection off asks first; turning it on does not. */
export function CaptureProtectionSetting({ settings, onChoose }: CaptureProtectionSettingProps) {
  const [isConfirmingOff, setIsConfirmingOff] = useState(false);
  const isSupported = settings.capture_protection_supported;
  const handleChange = (isOn: boolean) => (isOn ? onChoose(true) : setIsConfirmingOff(true));
  const confirmOff = () => {
    setIsConfirmingOff(false);
    onChoose(false);
  };
  return (
    <>
      <PrivacySwitch id="capture-protection" label="Block screen capture" isOn={settings.capture_protection && isSupported}
        isDisabled={!isSupported} onChange={handleChange} description={isSupported ? DESCRIPTION : UNSUPPORTED} />
      {isConfirmingOff && <CaptureOffConfirmation onTurnOff={confirmOff} onKeep={() => setIsConfirmingOff(false)} />}
    </>
  );
}
