import { useState, type FormEvent } from "react";
import { findPassphraseProblem } from "./passphrase-rules";

export function usePassphraseForm(requiresConfirmation: boolean, submit: (passphrase: string) => Promise<void>) {
  const [passphrase, setPassphrase] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    const problem = findPassphraseProblem(passphrase, confirmation, requiresConfirmation);
    setErrorMessage(problem);
    if (problem) return;
    setIsSubmitting(true);
    try {
      await submit(passphrase);
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setIsSubmitting(false);
    }
  };

  return { passphrase, setPassphrase, confirmation, setConfirmation, errorMessage, isSubmitting, handleSubmit };
}
