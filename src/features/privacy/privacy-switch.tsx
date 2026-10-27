import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

interface PrivacySwitchProps {
  id: string;
  label: string;
  description: string;
  isOn: boolean;
  isDisabled?: boolean;
  onChange: (isOn: boolean) => void;
}

/** One preference: a visible label, a switch, and a line saying what it does. */
export function PrivacySwitch({ id, label, description, isOn, isDisabled, onChange }: PrivacySwitchProps) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div className="flex flex-col gap-1">
        <Label htmlFor={id}>{label}</Label>
        <p id={`${id}-description`} className="text-xs text-muted-foreground">{description}</p>
      </div>
      <Switch id={id} checked={isOn} disabled={isDisabled} onCheckedChange={onChange} aria-describedby={`${id}-description`} />
    </div>
  );
}
