import { useState, type FormEvent } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { PasswordField } from "@/features/vault/password-field";
import { disableHardwareKeys } from "@/lib/api";

interface TurnOffFormProps {
  onDone: () => void;
  onCancel: () => void;
}

/** Stopping the key requirement lowers protection, so it asks for the passphrase and starts on the safe choice. */
export function TurnOffHardwareKeyForm({ onDone, onCancel }: TurnOffFormProps) {
  const [passphrase, setPassphrase] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    try {
      await disableHardwareKeys(passphrase);
      onDone();
    } catch (error) {
      setErrorMessage(String(error));
    }
  };

  return (
    <form onSubmit={submit} onKeyDown={(event) => event.key === "Escape" && onCancel()} className="flex flex-col gap-3">
      <p className="text-sm">Without the requirement, your passphrase alone opens the vault again. Your registered keys are forgotten.</p>
      <PasswordField id="turn-off-passphrase" label="Current passphrase" value={passphrase} onChange={setPassphrase} />
      {errorMessage && <Alert variant="destructive"><AlertDescription>{errorMessage}</AlertDescription></Alert>}
      <div className="flex gap-2">
        <Button type="button" variant="outline" autoFocus onClick={onCancel}>Keep requiring a key</Button>
        <Button type="submit" variant="destructive">Stop requiring a key</Button>
      </div>
    </form>
  );
}
