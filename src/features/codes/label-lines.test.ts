import { describe, expect, it } from "vitest";
import { labelLines } from "./label-lines";

describe("labelLines", () => {
  it("puts the service first and the account name under it", () => {
    expect(labelLines("GitHub", "me@example.com")).toEqual({ primary: "GitHub", secondary: "me@example.com" });
  });

  it("uses the account name alone when there is no service", () => {
    expect(labelLines("", "me@example.com")).toEqual({ primary: "me@example.com", secondary: null });
  });
});
