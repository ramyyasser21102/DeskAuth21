import { useVendors, vendorName } from "@/features/vendors/use-vendors";
import { filterByVendor, type VendorFilter } from "@/features/vendors/vendor-filter";
import { groupByVendor, matchesSearch, type VendorGroup } from "@/features/vendors/search-and-group";
import type { CodeView } from "@/lib/api";

/** What the list shows: the chosen app (or all), narrowed by the search, then grouped by app. */
export function useVisibleGroups(all: readonly CodeView[], filter: VendorFilter, query: string): VendorGroup[] {
  const vendors = useVendors();
  const matching = filterByVendor(all, filter).filter((view) => matchesSearch(view, query, vendorName(vendors, view.vendor)));
  return groupByVendor(matching, vendors);
}
