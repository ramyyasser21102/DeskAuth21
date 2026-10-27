import { useEffect, useState } from "react";
import { assessPassphrase, type PassphraseStrength } from "@/lib/api";

const TYPING_PAUSE_MS = 250;

/** The strength of what has been typed, asked for once the person pauses. `null` while there is nothing to rate. */
export function usePassphraseStrength(passphrase: string): PassphraseStrength | null {
  const [strength, setStrength] = useState<PassphraseStrength | null>(null);

  useEffect(() => {
    if (passphrase === "") return;
    let isCurrent = true;
    const timerId = window.setTimeout(() => {
      assessPassphrase(passphrase)
        .then((assessed) => isCurrent && setStrength(assessed))
        .catch(() => isCurrent && setStrength(null));
    }, TYPING_PAUSE_MS);
    return () => {
      isCurrent = false;
      window.clearTimeout(timerId);
    };
  }, [passphrase]);

  // The last rating stays on screen while typing continues; it only ever advises.
  return passphrase === "" ? null : strength;
}
