import { useCallback, useState } from "react";
import { saveRecoveryKey } from "@/lib/api";
import { CONFIRMATION_WORD_COUNT, confirmationMatches, pickConfirmationPositions, randomIndexBelow } from "./recovery-key-steps";
import { useGeneratedRecoveryKey } from "./use-generated-recovery-key";

export type WizardStep = "show" | "confirm" | "done";
const NOT_MATCHING = "Those words do not match the ones shown. Check them against what you wrote down.";

/** Makes a recovery key, makes the person prove they wrote it down, and only then saves it. */
export function useRecoveryKeyWizard() {
  const { words, problem } = useGeneratedRecoveryKey();
  const [step, setStep] = useState<WizardStep>("show");
  const [positions, setPositions] = useState<number[]>([]);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  const startConfirming = useCallback(() => {
    setPositions(pickConfirmationPositions(words.length, CONFIRMATION_WORD_COUNT, randomIndexBelow));
    setStep("confirm");
  }, [words.length]);

  const save = useCallback(async (typed: string[], currentPassphrase: string) => {
    if (!confirmationMatches(words, positions, typed)) return setSaveError(NOT_MATCHING);
    setIsBusy(true);
    try {
      await saveRecoveryKey(words.join(" "), currentPassphrase);
      setSaveError(null);
      setStep("done");
    } catch (error) {
      setSaveError(String(error));
    } finally {
      setIsBusy(false);
    }
  }, [words, positions]);

  return { step, words, positions, errorMessage: saveError ?? problem, isBusy, startConfirming, save };
}
