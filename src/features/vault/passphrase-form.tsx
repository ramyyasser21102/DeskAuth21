import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { PasswordField } from "./password-field";
import { usePassphraseForm } from "./use-passphrase-form";

export interface PassphraseFormProps {
  title: string;
  hint?: string;
  submitLabel: string;
  requiresConfirmation: boolean;
  onSubmit: (passphrase: string) => Promise<void>;
}

export function PassphraseForm({ title, hint, submitLabel, requiresConfirmation, onSubmit }: PassphraseFormProps) {
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
          {requiresConfirmation && (
            <PasswordField id="confirmation" label="Confirm passphrase" value={form.confirmation} onChange={form.setConfirmation} />
          )}
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
