import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { PassphraseGenerator } from "./passphrase-generator";
import { PassphraseStrengthMeter } from "./passphrase-strength-meter";
import { PasswordField } from "./password-field";
import { useRecoveryUnlock } from "./use-recovery-unlock";

const EXPLANATION = "Type the 24 words of your recovery key and choose a new passphrase. The old passphrase will stop working.";

export function RecoveryUnlockForm({ onUnlocked, onBack }: { onUnlocked: () => void; onBack: () => void }) {
  const form = useRecoveryUnlock(onUnlocked);
  return (
    <Card>
      <CardHeader>
        <CardTitle>Use your recovery key</CardTitle>
        <CardDescription>{EXPLANATION}</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={form.submit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="recovery-phrase">Recovery key (24 words)</Label>
            <Textarea id="recovery-phrase" rows={4} autoComplete="off" autoCapitalize="none" spellCheck={false} value={form.phrase} onChange={(event) => form.setPhrase(event.target.value)} autoFocus />
          </div>
          <PasswordField id="recovery-new-passphrase" label="New passphrase" value={form.passphrase} onChange={form.setPassphrase} />
          <PassphraseStrengthMeter passphrase={form.passphrase} />
          <PasswordField id="recovery-new-confirmation" label="Confirm new passphrase" value={form.confirmation} onChange={form.setConfirmation} />
          <PassphraseGenerator onGenerated={form.fillBoth} />
          {form.errorMessage && <Alert variant="destructive"><AlertDescription>{form.errorMessage}</AlertDescription></Alert>}
          <div className="flex gap-2">
            <Button type="submit" disabled={form.isBusy}>Reset passphrase and unlock</Button>
            <Button type="button" variant="outline" onClick={onBack}>Back to unlock</Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
