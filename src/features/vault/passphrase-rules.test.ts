import { describe, expect, it } from "vitest";
import { findPassphraseProblem, PASSPHRASE_MISMATCH_MESSAGE } from "./passphrase-rules";

describe("findPassphraseProblem", () => {
  it("reports a mismatch only when confirmation is required", () => {
    expect(findPassphraseProblem("abc", "abd", true)).toBe(PASSPHRASE_MISMATCH_MESSAGE);
    expect(findPassphraseProblem("abc", "abd", false)).toBeNull();
  });

  it("accepts matching passphrases", () => {
    expect(findPassphraseProblem("abc", "abc", true)).toBeNull();
  });
});
