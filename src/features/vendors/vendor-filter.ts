import type { CodeView, VendorId } from "@/lib/api";

export type VendorFilter = VendorId | "all";

export interface VendorCount {
  vendor: VendorId;
  count: number;
}

/** Vendors that have at least one account, most accounts first, ties in first-seen order. */
export function countByVendor(codes: readonly CodeView[]): VendorCount[] {
  const counts = new Map<VendorId, number>();
  for (const { vendor } of codes) counts.set(vendor, (counts.get(vendor) ?? 0) + 1);
  return [...counts].map(([vendor, count]) => ({ vendor, count })).sort((first, second) => second.count - first.count);
}

/** A chosen vendor with no accounts left (all deleted or moved) falls back to showing everything. */
export function resolveFilter(chosen: VendorFilter, counts: readonly VendorCount[]): VendorFilter {
  return chosen === "all" || counts.some(({ vendor }) => vendor === chosen) ? chosen : "all";
}

export function filterByVendor(codes: readonly CodeView[], filter: VendorFilter): CodeView[] {
  return filter === "all" ? [...codes] : codes.filter(({ vendor }) => vendor === filter);
}
