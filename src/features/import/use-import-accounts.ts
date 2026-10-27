import { useCallback, useState } from "react";
import { importUri, type ImportSummary } from "@/lib/api";
import { combineSummaries, describeImportOutcome } from "./import-result";
import { readQrTextFromImage } from "./qr-decoder";

export interface ImportOutcome {
  isError: boolean;
  message: string;
}

const errorText = (error: unknown) => (error instanceof Error ? error.message : String(error));

/** Imports each image in turn, collecting what was added and which images failed. */
async function importImages(files: readonly File[]) {
  const summaries: ImportSummary[] = [];
  const failures: string[] = [];
  for (const file of files) {
    try {
      summaries.push(await importUri(await readQrTextFromImage(file)));
    } catch (error) {
      failures.push(`${file.name || "pasted image"} (${errorText(error)})`);
    }
  }
  return { summary: combineSummaries(summaries), failures };
}

export function useImportAccounts() {
  const [outcome, setOutcome] = useState<ImportOutcome | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  const importFromImages = useCallback(async (files: File[]) => {
    setIsBusy(true);
    const { summary, failures } = await importImages(files);
    setOutcome({ isError: failures.length > 0, message: describeImportOutcome(summary, failures) });
    setIsBusy(false);
  }, []);

  const importFromLink = useCallback(async (link: string) => {
    setIsBusy(true);
    try {
      const summary = await importUri(link.trim());
      setOutcome({ isError: false, message: describeImportOutcome(summary, []) });
    } catch (error) {
      setOutcome({ isError: true, message: errorText(error) });
    } finally {
      setIsBusy(false);
    }
  }, []);

  return { outcome, isBusy, importFromImages, importFromLink };
}
