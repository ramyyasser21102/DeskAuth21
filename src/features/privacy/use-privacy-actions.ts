import { useState } from "react";
import { setCaptureProtection, setHideCodes, setTrayShowsCodes, type Settings } from "@/lib/api";
import { usePrivacySettings } from "./privacy-settings-provider";

/** Saves a privacy choice and shares the result, or keeps a message saying why it failed. */
export function usePrivacyActions() {
  const { applySaved } = usePrivacySettings();
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const save = async (change: () => Promise<Settings>) => {
    try {
      applySaved(await change());
      setErrorMessage(null);
    } catch (error) {
      setErrorMessage(String(error));
    }
  };

  return {
    errorMessage,
    chooseCaptureProtection: (enabled: boolean) => save(() => setCaptureProtection(enabled)),
    chooseHiddenCodes: (hidden: boolean) => save(() => setHideCodes(hidden)),
    chooseTrayCodes: (shown: boolean) => save(() => setTrayShowsCodes(shown)),
  };
}
