import { describe, expect, it } from "vitest";
import { AUTO_LOCK_OPTIONS, describeAutoLock } from "./auto-lock-options";

describe("describeAutoLock", () => {
  it("uses singular and plural minutes", () => {
    expect(describeAutoLock(1)).toBe("The vault locks after 1 minute without activity.");
    expect(describeAutoLock(15)).toBe("The vault locks after 15 minutes without activity.");
  });

  it("explains the never option", () => {
    expect(describeAutoLock(null)).toBe("The vault stays unlocked until you lock it.");
  });
});

describe("AUTO_LOCK_OPTIONS", () => {
  it("offers each choice once, ending with Never", () => {
    const minutes = AUTO_LOCK_OPTIONS.map((option) => option.minutes);
    expect(new Set(minutes).size).toBe(minutes.length);
    expect(minutes[minutes.length - 1]).toBeNull();
  });
});
