import type { ImportSummary } from "@/lib/api";

export const EMPTY_SUMMARY: ImportSummary = { added: 0, skipped_duplicates: 0, skipped_unsupported: 0 };

export function combineSummaries(summaries: readonly ImportSummary[]): ImportSummary {
  return summaries.reduce(
    (total, next) => ({
      added: total.added + next.added,
      skipped_duplicates: total.skipped_duplicates + next.skipped_duplicates,
      skipped_unsupported: total.skipped_unsupported + next.skipped_unsupported,
    }),
    EMPTY_SUMMARY,
  );
}

const pluralize = (count: number, singular: string, plural = `${singular}s`) => `${count} ${count === 1 ? singular : plural}`;

function describeImported({ added, skipped_duplicates }: ImportSummary): string {
  if (added === 0 && skipped_duplicates === 0) return "No accounts were found.";
  if (added === 0) return `Nothing new: ${pluralize(skipped_duplicates, "account")} already in your vault.`;
  const addedText = `Added ${pluralize(added, "account")}.`;
  return skipped_duplicates === 0 ? addedText : `${addedText} Skipped ${skipped_duplicates} already in your vault.`;
}

function describeSummary(summary: ImportSummary): string {
  const { skipped_unsupported } = summary;
  if (skipped_unsupported === 0) return describeImported(summary);
  const unsupported = `${pluralize(skipped_unsupported, "entry", "entries")} could not be imported (counter-based, Steam or unreadable).`;
  const nothingImported = summary.added + summary.skipped_duplicates === 0;
  return nothingImported ? unsupported : `${describeImported(summary)} ${unsupported}`;
}

/** One message covering both what was imported and which sources failed. */
export function describeImportOutcome(summary: ImportSummary, failures: readonly string[]): string {
  const failureText = failures.map((failure) => `Could not import ${failure}.`);
  const hasImportedAnything = summary.added + summary.skipped_duplicates + summary.skipped_unsupported > 0;
  const summaryText = hasImportedAnything || failures.length === 0 ? [describeSummary(summary)] : [];
  return [...summaryText, ...failureText].join(" ");
}
