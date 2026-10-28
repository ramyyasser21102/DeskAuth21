import { PrivacySwitch } from "@/features/privacy/privacy-switch";
import { useLaunchAtLogin } from "./use-launch-at-login";

const DESCRIPTION = "Starts DA21 hidden in the menu bar or tray when you sign in, so your codes are one click away.";

export function LaunchAtLoginSetting() {
  const { isOn, errorMessage, choose } = useLaunchAtLogin();
  if (isOn === null) return null;
  return (
    <div className="flex flex-col gap-2">
      <PrivacySwitch id="launch-at-login" label="Launch at login" isOn={isOn} onChange={(wanted) => void choose(wanted)} description={DESCRIPTION} />
      {errorMessage && <p role="alert" className="text-xs text-destructive">{errorMessage}</p>}
    </div>
  );
}
