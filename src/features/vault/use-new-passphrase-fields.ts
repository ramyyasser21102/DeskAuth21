import { useState } from "react";

/** The two fields for choosing a passphrase, and filling both from a generated one. */
export function useNewPassphraseFields() {
  const [passphrase, setPassphrase] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const fillBoth = (generated: string) => {
    setPassphrase(generated);
    setConfirmation(generated);
  };
  const clear = () => fillBoth("");
  return { passphrase, setPassphrase, confirmation, setConfirmation, fillBoth, clear };
}
