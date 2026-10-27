import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { PassphraseGenerator } from "@/features/vault/passphrase-generator";
import { PassphraseStrengthMeter } from "@/features/vault/passphrase-strength-meter";
import { PasswordField } from "@/features/vault/password-field";
import { useChangePassphrase } from "./use-change-passphrase";

export function ChangePassphraseForm({ onClose }: { onClose: () => void }) {
  const form = useChangePassphrase();
  return (
    <form onSubmit={form.submit} className="flex flex-col gap-3">
      <PasswordField id="change-current" label="Current passphrase" value={form.current} onChange={form.setCurrent} autoFocus />
      <PasswordField id="change-new" label="New passphrase" value={form.passphrase} onChange={form.setPassphrase} />
      <PassphraseStrengthMeter passphrase={form.passphrase} />
      <PasswordField id="change-confirm" label="Confirm new passphrase" value={form.confirmation} onChange={form.setConfirmation} />
      <PassphraseGenerator onGenerated={form.fillBoth} />
      {form.errorMessage && <Alert variant="destructive"><AlertDescription>{form.errorMessage}</AlertDescription></Alert>}
      {form.didChange && <p role="status" className="text-sm">Passphrase changed. Use the new one next time you unlock.</p>}
      <div className="flex gap-2">
        <Button type="submit" disabled={form.isBusy}>Change passphrase</Button>
        <Button type="button" variant="outline" onClick={onClose}>{form.didChange ? "Close" : "Cancel"}</Button>
      </div>
    </form>
  );
}
