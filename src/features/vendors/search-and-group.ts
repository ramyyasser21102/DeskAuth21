import type { CodeView, VendorId, VendorInfo } from "@/lib/api";
import { vendorName } from "./use-vendors";

export interface VendorGroup {
  vendor: VendorId;
  name: string;
  views: CodeView[];
}

const OTHER_VENDOR: VendorId = "other";

/** Every word typed must appear somewhere in the service, the account name or the app's name. */
export function matchesSearch(view: CodeView, query: string, sourceName: string): boolean {
  const terms = query.toLowerCase().split(/\s+/).filter((term) => term !== "");
  const haystack = `${view.issuer} ${view.name} ${sourceName}`.toLowerCase();
  return terms.every((term) => haystack.includes(term));
}

const byLabel = (first: CodeView, second: CodeView) => first.label.localeCompare(second.label, undefined, { sensitivity: "base" });

/**
 * Accounts under the app they came from: apps A to Z with "Other" last, accounts A to Z
 * within each, so a given account is always in the same place.
 */
export function groupByVendor(views: readonly CodeView[], vendors: readonly VendorInfo[]): VendorGroup[] {
  const groups = new Map<VendorId, CodeView[]>();
  for (const view of views) groups.set(view.vendor, [...(groups.get(view.vendor) ?? []), view]);
  return [...groups]
    .map(([vendor, members]) => ({ vendor, name: vendorName(vendors, vendor), views: [...members].sort(byLabel) }))
    .sort((first, second) => compareVendors(first, second));
}

function compareVendors(first: VendorGroup, second: VendorGroup): number {
  if (first.vendor === OTHER_VENDOR) return second.vendor === OTHER_VENDOR ? 0 : 1;
  if (second.vendor === OTHER_VENDOR) return -1;
  return first.name.localeCompare(second.name, undefined, { sensitivity: "base" });
}
