import { Button } from "@/components/ui/button";
import { HardwareKeyForm } from "./hardware-key-form";
import { TurnOffHardwareKeyForm } from "./turn-off-hardware-key-form";

export type Panel = "none" | "enable" | "add" | "turn-off";

interface PanelProps {
  panel: Panel;
  onDone: () => void;
  onCancel: () => void;
}

/** Whichever form is open, if any. */
export function ActivePanel({ panel, onDone, onCancel }: PanelProps) {
  if (panel === "enable" || panel === "add") return <HardwareKeyForm mode={panel} onDone={onDone} onCancel={onCancel} />;
  if (panel === "turn-off") return <TurnOffHardwareKeyForm onDone={onDone} onCancel={onCancel} />;
  return null;
}

interface ButtonsProps {
  isRequired: boolean;
  onOpen: (panel: Panel) => void;
}

export function PanelButtons({ isRequired, onOpen }: ButtonsProps) {
  if (!isRequired) {
    return <Button type="button" variant="outline" size="sm" className="self-start" onClick={() => onOpen("enable")}>Require a hardware key</Button>;
  }
  return (
    <div className="flex flex-wrap gap-2">
      <Button type="button" variant="outline" size="sm" onClick={() => onOpen("add")}>Add another key</Button>
      <Button type="button" variant="ghost" size="sm" onClick={() => onOpen("turn-off")}>Stop requiring a key</Button>
    </div>
  );
}
