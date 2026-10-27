import { useState, type FormEvent } from "react";
import { addHardwareKey, enableHardwareKey } from "@/lib/api";

export type KeyFormMode = "enable" | "add";
const DEFAULT_LABEL = "Hardware key";

/** Fields and submit for registering a key; the backend waits for the touch, so `isBusy` can last a while. */
export function useHardwareKeyForm(mode: KeyFormMode, onDone: () => void) {
  const [label, setLabel] = useState(DEFAULT_LABEL);
  const [pin, setPin] = useState("");
  const [passphrase, setPassphrase] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setIsBusy(true);
    setErrorMessage(null);
    const keyPin = pin === "" ? null : pin;
    try {
      await (mode === "enable" ? enableHardwareKey(passphrase, keyPin, label.trim()) : addHardwareKey(keyPin, label.trim()));
      onDone();
    } catch (error) {
      setErrorMessage(String(error));
      setIsBusy(false);
    }
  };

  return { label, setLabel, pin, setPin, passphrase, setPassphrase, errorMessage, isBusy, submit };
}
