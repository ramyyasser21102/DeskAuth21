import { useState } from "react";
import { HardwareKeyList } from "./hardware-key-list";
import { ActivePanel, PanelButtons, type Panel } from "./hardware-key-panels";
import { useHardwareKeys } from "./use-hardware-keys";

const WHY = "A hardware key (a security key such as a YubiKey) can be required as well as your passphrase, so a copied vault file cannot be opened without it.";
const UNSUPPORTED = "Hardware keys are not supported on this system yet.";

export function HardwareKeySetting() {
  const { status, refresh } = useHardwareKeys();
  const [panel, setPanel] = useState<Panel>("none");
  if (!status) return null;
  const finish = () => {
    setPanel("none");
    refresh();
  };
  const canShowButtons = status.is_supported && panel === "none";

  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium">Hardware key: {status.is_required ? "required" : "not required"}</span>
      <p className="text-xs text-muted-foreground">{status.is_supported ? WHY : UNSUPPORTED}</p>
      <ActivePanel panel={panel} onDone={finish} onCancel={() => setPanel("none")} />
      {status.is_required && <HardwareKeyList keys={status.keys} onChanged={refresh} />}
      {canShowButtons && <PanelButtons isRequired={status.is_required} onOpen={setPanel} />}
    </div>
  );
}
