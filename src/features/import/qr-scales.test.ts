import { describe, expect, it, vi } from "vitest";
import { firstQrText, planScales } from "./qr-scales";

describe("planScales", () => {
  it("tries the image itself first, then enlarged versions of small images", () => {
    expect(planScales(400, 400)).toEqual([1, 2, 3]);
  });

  it("stops enlarging once the image would get too big", () => {
    expect(planScales(2880, 1800)).toEqual([1, 2]);
    expect(planScales(3840, 2160)).toEqual([1, 2]);
  });

  it("shrinks enormous images first and still tries something", () => {
    const [first] = planScales(12000, 8000);
    expect(first).toBeLessThan(1);
    expect(planScales(12000, 8000).length).toBeGreaterThan(0);
  });
});

describe("firstQrText", () => {
  it("returns the first success and stops trying", async () => {
    const decodeAt = vi.fn(async (scale: number) => (scale >= 2 ? "found" : null));
    expect(await firstQrText([1, 2, 3], decodeAt)).toBe("found");
    expect(decodeAt).toHaveBeenCalledTimes(2);
  });

  it("returns null when no scale works", async () => {
    expect(await firstQrText([1, 2], async () => null)).toBeNull();
  });
});
