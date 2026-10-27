import type { ReactNode } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { PassphraseGenerator } from "./passphrase-generator";
import { PassphraseStrengthMeter } from "./passphrase-strength-meter";
import { PasswordField } from "./password-field";
import { usePassphraseForm } from "./use-passphrase-form";

export interface PassphraseFormProps {
  title: string;
  hint?: string;
  submitLabel: string;
  requiresConfirmation: boolean;
  onSubmit: (passphrase: string) => Promise<void>;
  /** Extra fields, shown after the passphrase (for example a security key's PIN). */
  children?: ReactNode;
  /** Shown while the form is working, for steps that wait on the person (touching a key). */
  busyMessage?: string;
}

export function PassphraseForm({ title, hint, submitLabel, requiresConfirmation, onSubmit, children, busyMessage }: PassphraseFormProps) {
  const form = usePassphraseForm(requiresConfirmation, onSubmit);
  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {hint && <CardDescription>{hint}</CardDescription>}
      </CardHeader>
      <CardContent>
        <form onSubmit={form.handleSubmit} className="flex flex-col gap-4">
          <PasswordField id="passphrase" label="Passphrase" value={form.passphrase} onChange={form.setPassphrase} autoFocus />
          {requiresConfirmation && <PassphraseStrengthMeter passphrase={form.passphrase} />}
          {requiresConfirmation && (
            <PasswordField id="confirmation" label="Confirm passphrase" value={form.confirmation} onChange={form.setConfirmation} />
          )}
          {requiresConfirmation && <PassphraseGenerator onGenerated={(phrase) => { form.setPassphrase(phrase); form.setConfirmation(phrase); }} />}
          {children}
          {form.isSubmitting && busyMessage && <p role="status" className="text-sm font-medium">{busyMessage}</p>}
          {form.errorMessage && (
            <Alert variant="destructive">
              <AlertDescription>{form.errorMessage}</AlertDescription>
            </Alert>
          )}
          <Button type="submit" disabled={form.isSubmitting}>{submitLabel}</Button>
        </form>
      </CardContent>
    </Card>
  );
}
