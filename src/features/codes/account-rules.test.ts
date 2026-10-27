import { describe, expect, it } from "vitest";
import { EMPTY_NAME_MESSAGE, findNameProblem } from "./account-rules";

describe("findNameProblem", () => {
  it("rejects empty and whitespace-only names", () => {
    expect(findNameProblem("")).toBe(EMPTY_NAME_MESSAGE);
    expect(findNameProblem("   ")).toBe(EMPTY_NAME_MESSAGE);
  });

  it("accepts a real name", () => {
    expect(findNameProblem("me@example.com")).toBeNull();
  });
});
