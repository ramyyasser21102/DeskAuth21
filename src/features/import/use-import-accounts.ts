import { useCallback, useState } from "react";
import { importText, type ImportSummary, type VendorId } from "@/lib/api";
import { combineSummaries, describeImportOutcome } from "./import-result";
import { readImportText } from "./read-import-text";

export interface ImportOutcome {
  isError: boolean;
  message: string;
}

const errorText = (error: unknown) => (error instanceof Error ? error.message : String(error));

/** Imports each file in turn (a QR image or an export file), collecting what was added and which failed. */
async function importFiles(files: readonly File[], vendor: VendorId) {
  const summaries: ImportSummary[] = [];
  const failures: string[] = [];
  for (const file of files) {
    try {
      summaries.push(await importText(await readImportText(file), vendor));
    } catch (error) {
      failures.push(`${file.name || "pasted image"} (${errorText(error)})`);
    }
  }
  return { summary: combineSummaries(summaries), failures };
}

/** `vendor` tags accounts whose file does not say which app they came from. */
export function useImportAccounts(vendor: VendorId) {
  const [outcome, setOutcome] = useState<ImportOutcome | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  const importFromFiles = useCallback(async (files: File[]) => {
    setIsBusy(true);
    const { summary, failures } = await importFiles(files, vendor);
    setOutcome({ isError: failures.length > 0, message: describeImportOutcome(summary, failures) });
    setIsBusy(false);
  }, [vendor]);

  const importFromLink = useCallback(async (link: string) => {
    setIsBusy(true);
    try {
      const summary = await importText(link.trim(), vendor);
      setOutcome({ isError: false, message: describeImportOutcome(summary, []) });
    } catch (error) {
      setOutcome({ isError: true, message: errorText(error) });
    } finally {
      setIsBusy(false);
    }
  }, [vendor]);

  return { outcome, isBusy, importFromFiles, importFromLink };
}
