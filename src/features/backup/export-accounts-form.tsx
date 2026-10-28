import { useState, type FormEvent } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { PasswordField } from "@/features/vault/password-field";
import { exportAccountsFile } from "@/lib/api";
import { describeExportResult } from "./backup-messages";

interface ExportAccountsFormProps {
  onDone: (message: string) => void;
  onCancel: () => void;
}

const WARNING = "The file holds every account's secret key, unencrypted. Anyone who gets it can make your codes. Keep it somewhere private and delete it when you are done.";

/** Plain-text export is the one place secrets leave the vault, so it asks for the passphrase again and starts on the safe choice. */
export function ExportAccountsForm({ onDone, onCancel }: ExportAccountsFormProps) {
  const [passphrase, setPassphrase] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    try {
      const written = await exportAccountsFile(passphrase);
      if (written === null) return onCancel();
      onDone(describeExportResult(written));
    } catch (error) {
      setErrorMessage(String(error));
    }
  };

  return (
    <form onSubmit={submit} onKeyDown={(event) => event.key === "Escape" && onCancel()} className="flex flex-col gap-3">
      <p className="text-sm">{WARNING}</p>
      <PasswordField id="export-passphrase" label="Current passphrase" value={passphrase} onChange={setPassphrase} />
      {errorMessage && <Alert variant="destructive"><AlertDescription>{errorMessage}</AlertDescription></Alert>}
      <div className="flex gap-2">
        <Button type="button" variant="outline" autoFocus onClick={onCancel}>Cancel</Button>
        <Button type="submit" variant="destructive">Choose where to save…</Button>
      </div>
    </form>
  );
}
