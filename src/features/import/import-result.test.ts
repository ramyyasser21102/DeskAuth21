import { describe, expect, it } from "vitest";
import { combineSummaries, describeImportOutcome, EMPTY_SUMMARY } from "./import-result";

const summary = (added: number, skipped_duplicates: number) => ({ added, skipped_duplicates });

describe("combineSummaries", () => {
  it("adds up several imports", () => {
    expect(combineSummaries([summary(2, 1), summary(3, 0)])).toEqual(summary(5, 1));
    expect(combineSummaries([])).toEqual(EMPTY_SUMMARY);
  });
});

describe("describeImportOutcome", () => {
  it("describes additions with correct plurals", () => {
    expect(describeImportOutcome(summary(1, 0), [])).toBe("Added 1 account.");
    expect(describeImportOutcome(summary(3, 0), [])).toBe("Added 3 accounts.");
  });

  it("mentions skipped duplicates", () => {
    expect(describeImportOutcome(summary(2, 1), [])).toBe("Added 2 accounts. Skipped 1 already in your vault.");
    expect(describeImportOutcome(summary(0, 2), [])).toBe("Nothing new: 2 accounts already in your vault.");
  });

  it("reports failures, with or without partial success", () => {
    expect(describeImportOutcome(summary(1, 0), ["b.png (no QR code found)"])).toBe(
      "Added 1 account. Could not import b.png (no QR code found).",
    );
    expect(describeImportOutcome(EMPTY_SUMMARY, ["a.png (no QR code found)"])).toBe(
      "Could not import a.png (no QR code found).",
    );
  });

  it("says so when nothing was found", () => {
    expect(describeImportOutcome(EMPTY_SUMMARY, [])).toBe("No accounts were found.");
  });
});
