import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PasswordField } from "@/features/vault/password-field";
import { useHardwareKeyForm, type KeyFormMode } from "./use-hardware-key-form";

const INTRO: Record<KeyFormMode, string> = {
  enable: "From then on the vault needs this key as well as your passphrase. You need a recovery key first, in case the key is lost. Plug the key in; you will be asked to touch it twice.",
  add: "Plug in another key; you will be asked to touch it twice. Either key will then open the vault.",
};
const SUBMIT: Record<KeyFormMode, string> = { enable: "Register key and require it", add: "Register key" };

interface HardwareKeyFormProps {
  mode: KeyFormMode;
  onDone: () => void;
  onCancel: () => void;
}

export function HardwareKeyForm({ mode, onDone, onCancel }: HardwareKeyFormProps) {
  const form = useHardwareKeyForm(mode, onDone);
  return (
    <form onSubmit={form.submit} className="flex flex-col gap-3">
      <p className="text-sm">{INTRO[mode]}</p>
      <div className="flex flex-col gap-1">
        <Label htmlFor="hardware-key-label">Key name</Label>
        <Input id="hardware-key-label" value={form.label} maxLength={32} onChange={(event) => form.setLabel(event.target.value)} />
      </div>
      <PasswordField id="hardware-key-pin" label="Key PIN (only if your key has one)" value={form.pin} onChange={form.setPin} />
      {mode === "enable" && <PasswordField id="hardware-key-passphrase" label="Current passphrase" value={form.passphrase} onChange={form.setPassphrase} />}
      {form.isBusy && <p role="status" className="text-sm font-medium">Touch your key when it blinks…</p>}
      {form.errorMessage && <Alert variant="destructive"><AlertDescription>{form.errorMessage}</AlertDescription></Alert>}
      <div className="flex gap-2">
        <Button type="submit" disabled={form.isBusy}>{SUBMIT[mode]}</Button>
        <Button type="button" variant="outline" disabled={form.isBusy} onClick={onCancel}>Cancel</Button>
      </div>
    </form>
  );
}
