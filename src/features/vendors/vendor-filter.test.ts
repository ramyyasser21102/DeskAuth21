import { describe, expect, it } from "vitest";
import type { CodeView, VendorId } from "@/lib/api";
import { countByVendor, filterByVendor, resolveFilter } from "./vendor-filter";

const account = (id: string, vendor: VendorId): CodeView => ({
  id, label: id, issuer: id, name: id, vendor, code: "123456", seconds_left: 10, period_seconds: 30,
});
const CODES = [account("a", "aegis"), account("b", "authy"), account("c", "authy"), account("d", "other")];

describe("countByVendor", () => {
  it("lists only vendors in use, biggest first", () => {
    expect(countByVendor(CODES)).toEqual([
      { vendor: "authy", count: 2 },
      { vendor: "aegis", count: 1 },
      { vendor: "other", count: 1 },
    ]);
  });

  it("is empty for no accounts", () => {
    expect(countByVendor([])).toEqual([]);
  });
});

describe("filterByVendor", () => {
  it("keeps everything for all", () => {
    expect(filterByVendor(CODES, "all")).toHaveLength(4);
  });

  it("keeps only the chosen vendor, in order", () => {
    expect(filterByVendor(CODES, "authy").map(({ id }) => id)).toEqual(["b", "c"]);
  });
});

describe("resolveFilter", () => {
  const counts = countByVendor(CODES);

  it("keeps a vendor that still has accounts", () => {
    expect(resolveFilter("aegis", counts)).toBe("aegis");
  });

  it("falls back to all when the vendor has no accounts", () => {
    expect(resolveFilter("bitwarden", counts)).toBe("all");
  });
});
