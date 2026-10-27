import { describe, expect, it } from "vitest";
import { dashOffsetFor, EXPIRING_SOON_SECONDS, fractionRemaining, isExpiringSoon } from "./countdown";

describe("fractionRemaining", () => {
  it("is the share of the window left", () => {
    expect(fractionRemaining(15, 30)).toBe(0.5);
    expect(fractionRemaining(30, 30)).toBe(1);
    expect(fractionRemaining(15, 60)).toBe(0.25);
  });

  it("stays between 0 and 1 and survives a zero period", () => {
    expect(fractionRemaining(-3, 30)).toBe(0);
    expect(fractionRemaining(45, 30)).toBe(1);
    expect(fractionRemaining(10, 0)).toBe(0);
  });
});

describe("isExpiringSoon", () => {
  it("turns on at the threshold", () => {
    expect(isExpiringSoon(EXPIRING_SOON_SECONDS)).toBe(true);
    expect(isExpiringSoon(EXPIRING_SOON_SECONDS + 1)).toBe(false);
  });
});

describe("dashOffsetFor", () => {
  it("draws the whole outline when full and none when empty", () => {
    expect(dashOffsetFor(100, 1)).toBe(0);
    expect(dashOffsetFor(100, 0)).toBe(100);
    expect(dashOffsetFor(100, 0.25)).toBe(75);
  });
});
