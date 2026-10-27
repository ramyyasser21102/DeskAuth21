import { describe, expect, it } from "vitest";
import { confirmationMatches, pickConfirmationPositions, splitWords } from "./recovery-key-steps";

describe("pickConfirmationPositions", () => {
  it("returns distinct positions, lowest first, redrawing repeats", () => {
    const draws = [7, 7, 2, 19];
    const positions = pickConfirmationPositions(24, 3, () => draws.shift() ?? 0);
    expect(positions).toEqual([2, 7, 19]);
  });
});

describe("confirmationMatches", () => {
  const words = ["alpha", "bravo", "charlie", "delta"];

  it("accepts the right words in any case with stray spaces", () => {
    expect(confirmationMatches(words, [1, 3], [" BRAVO", "delta "])).toBe(true);
  });

  it("refuses a wrong or missing word", () => {
    expect(confirmationMatches(words, [1, 3], ["bravo", "echo"])).toBe(false);
    expect(confirmationMatches(words, [1, 3], ["bravo"])).toBe(false);
  });
});

describe("splitWords", () => {
  it("splits on any whitespace and ignores the ends", () => {
    expect(splitWords("  one two\nthree ")).toEqual(["one", "two", "three"]);
    expect(splitWords("")).toEqual([]);
  });
});
