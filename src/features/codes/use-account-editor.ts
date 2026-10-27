import { useState } from "react";
import { deleteAccount, renameAccount, type CodeView } from "@/lib/api";
import { findNameProblem } from "./account-rules";

export type EditorStep = "editing" | "confirming-delete";

/** State and actions for editing one account's labels, or deleting it after a confirmation. */
export function useAccountEditor(view: CodeView, onDone: () => void) {
  const [issuer, setIssuer] = useState(view.issuer);
  const [name, setName] = useState(view.name);
  const [step, setStep] = useState<EditorStep>("editing");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  const run = async (action: () => Promise<void>) => {
    setIsBusy(true);
    setErrorMessage(null);
    try {
      await action();
      onDone();
    } catch (error) {
      setErrorMessage(String(error));
      setIsBusy(false);
    }
  };

  const save = async () => {
    const problem = findNameProblem(name);
    if (problem) return setErrorMessage(problem);
    await run(() => renameAccount(view.id, { issuer, name }));
  };

  const confirmDelete = () => run(() => deleteAccount(view.id));
  const askToDelete = () => setStep("confirming-delete");
  const cancelDelete = () => setStep("editing");

  return { issuer, setIssuer, name, setName, step, errorMessage, isBusy, save, askToDelete, cancelDelete, confirmDelete };
}
