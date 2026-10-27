import type { ImportSummary } from "@/lib/api";

export const EMPTY_SUMMARY: ImportSummary = { added: 0, skipped_duplicates: 0 };

export function combineSummaries(summaries: readonly ImportSummary[]): ImportSummary {
  return summaries.reduce(
    (total, next) => ({
      added: total.added + next.added,
      skipped_duplicates: total.skipped_duplicates + next.skipped_duplicates,
    }),
    EMPTY_SUMMARY,
  );
}

const pluralize = (count: number, noun: string) => `${count} ${noun}${count === 1 ? "" : "s"}`;

function describeSummary({ added, skipped_duplicates }: ImportSummary): string {
  if (added === 0 && skipped_duplicates === 0) return "No accounts were found.";
  if (added === 0) return `Nothing new: ${pluralize(skipped_duplicates, "account")} already in your vault.`;
  const addedText = `Added ${pluralize(added, "account")}.`;
  return skipped_duplicates === 0 ? addedText : `${addedText} Skipped ${skipped_duplicates} already in your vault.`;
}

/** One message covering both what was imported and which sources failed. */
export function describeImportOutcome(summary: ImportSummary, failures: readonly string[]): string {
  const failureText = failures.map((failure) => `Could not import ${failure}.`);
  const hasImportedAnything = summary.added + summary.skipped_duplicates > 0;
  const summaryText = hasImportedAnything || failures.length === 0 ? [describeSummary(summary)] : [];
  return [...summaryText, ...failureText].join(" ");
}
