import { useState, type FormEvent } from "react";
import { resetPassphraseWithRecoveryKey } from "@/lib/api";
import { findPassphraseProblem } from "./passphrase-rules";
import { useNewPassphraseFields } from "./use-new-passphrase-fields";

/** Opens the vault with the recovery key and sets the new passphrase in one step. */
export function useRecoveryUnlock(onUnlocked: () => void) {
  const [phrase, setPhrase] = useState("");
  const fields = useNewPassphraseFields();
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const problem = findPassphraseProblem(fields.passphrase, fields.confirmation, true);
    setErrorMessage(problem);
    if (problem) return;
    setIsBusy(true);
    try {
      await resetPassphraseWithRecoveryKey(phrase, fields.passphrase);
      onUnlocked();
    } catch (error) {
      setErrorMessage(String(error));
      setIsBusy(false);
    }
  };

  return { phrase, setPhrase, ...fields, errorMessage, isBusy, submit };
}
