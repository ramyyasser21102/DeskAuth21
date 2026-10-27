import { useState, type FormEvent } from "react";
import { changePassphrase } from "@/lib/api";
import { findPassphraseProblem } from "@/features/vault/passphrase-rules";
import { useNewPassphraseFields } from "@/features/vault/use-new-passphrase-fields";

/** Fields and submit for changing the passphrase; the backend checks the current one. */
export function useChangePassphrase() {
  const [current, setCurrent] = useState("");
  const fields = useNewPassphraseFields();
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [didChange, setDidChange] = useState(false);
  const [isBusy, setIsBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setDidChange(false);
    const problem = findPassphraseProblem(fields.passphrase, fields.confirmation, true);
    setErrorMessage(problem);
    if (problem) return;
    setIsBusy(true);
    try {
      await changePassphrase(current, fields.passphrase);
      setCurrent("");
      fields.clear();
      setDidChange(true);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setIsBusy(false);
    }
  };

  return { current, setCurrent, ...fields, errorMessage, didChange, isBusy, submit };
}
