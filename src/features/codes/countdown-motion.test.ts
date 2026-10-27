import { describe, expect, it } from "vitest";
import { ringSweep } from "./countdown-motion";

describe("ringSweep", () => {
  it("drains steadily for a full second", () => {
    expect(ringSweep(0.5, 0.4, false)).toEqual({ duration: 1, ease: "linear" });
  });

  it("refills quickly when a new code starts", () => {
    expect(ringSweep(0.03, 1, false).duration).toBe(0.3);
  });

  it("jumps with no sweep when motion is reduced", () => {
    expect(ringSweep(0.03, 1, true)).toEqual({ duration: 0 });
  });
});
