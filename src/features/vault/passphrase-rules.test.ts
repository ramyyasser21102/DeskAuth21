import { describe, expect, it } from "vitest";
import { countCharacters, findPassphraseProblem, MIN_PASSPHRASE_CHARS, PASSPHRASE_MISMATCH_MESSAGE, PASSPHRASE_TOO_SHORT_MESSAGE } from "./passphrase-rules";

const LONG_ENOUGH = "a".repeat(MIN_PASSPHRASE_CHARS);

describe("findPassphraseProblem when choosing a new passphrase", () => {
  it("asks for more characters below the minimum", () => {
    expect(findPassphraseProblem("a".repeat(MIN_PASSPHRASE_CHARS - 1), "x", true)).toBe(PASSPHRASE_TOO_SHORT_MESSAGE);
  });

  it("reports a mismatch once the length is fine", () => {
    expect(findPassphraseProblem(LONG_ENOUGH, `${LONG_ENOUGH}!`, true)).toBe(PASSPHRASE_MISMATCH_MESSAGE);
  });

  it("accepts a long enough passphrase that matches", () => {
    expect(findPassphraseProblem(LONG_ENOUGH, LONG_ENOUGH, true)).toBeNull();
  });
});

describe("findPassphraseProblem when unlocking", () => {
  it("accepts whatever the existing vault was made with", () => {
    expect(findPassphraseProblem("short", "", false)).toBeNull();
  });
});

describe("countCharacters", () => {
  it("counts a character once however it is encoded", () => {
    expect(countCharacters("éé")).toBe(2);
    expect(countCharacters("🙂🙂")).toBe(2);
  });
});
