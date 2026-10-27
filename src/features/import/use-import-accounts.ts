import { useCallback, useState } from "react";
import { importText, type ImportSummary } from "@/lib/api";
import { combineSummaries, describeImportOutcome } from "./import-result";
import { readImportText } from "./read-import-text";

export interface ImportOutcome {
  isError: boolean;
  message: string;
}

const errorText = (error: unknown) => (error instanceof Error ? error.message : String(error));

/** Imports each file in turn (a QR image or an export file), collecting what was added and which failed. */
async function importFiles(files: readonly File[]) {
  const summaries: ImportSummary[] = [];
  const failures: string[] = [];
  for (const file of files) {
    try {
      summaries.push(await importText(await readImportText(file)));
    } catch (error) {
      failures.push(`${file.name || "pasted image"} (${errorText(error)})`);
    }
  }
  return { summary: combineSummaries(summaries), failures };
}

export function useImportAccounts() {
  const [outcome, setOutcome] = useState<ImportOutcome | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  const importFromFiles = useCallback(async (files: File[]) => {
    setIsBusy(true);
    const { summary, failures } = await importFiles(files);
    setOutcome({ isError: failures.length > 0, message: describeImportOutcome(summary, failures) });
    setIsBusy(false);
  }, []);

  const importFromLink = useCallback(async (link: string) => {
    setIsBusy(true);
    try {
      const summary = await importText(link.trim());
      setOutcome({ isError: false, message: describeImportOutcome(summary, []) });
    } catch (error) {
      setOutcome({ isError: true, message: errorText(error) });
    } finally {
      setIsBusy(false);
    }
  }, []);

  return { outcome, isBusy, importFromFiles, importFromLink };
}
