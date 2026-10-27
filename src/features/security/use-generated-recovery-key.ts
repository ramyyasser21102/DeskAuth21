import { useEffect, useState } from "react";
import { generateRecoveryKey } from "@/lib/api";
import { splitWords } from "./recovery-key-steps";

/** A fresh, unsaved recovery key as a list of words, made once when it is first needed. */
export function useGeneratedRecoveryKey() {
  const [words, setWords] = useState<string[]>([]);
  const [problem, setProblem] = useState<string | null>(null);
  useEffect(() => {
    let isActive = true;
    generateRecoveryKey()
      .then((phrase) => isActive && setWords(splitWords(phrase)))
      .catch((error) => isActive && setProblem(String(error)));
    return () => {
      isActive = false;
    };
  }, []);
  return { words, problem };
}
