import { groupByVendor, matchesSearch, type VendorGroup } from "@/features/vendors/search-and-group";
import { useVendors, vendorName } from "@/features/vendors/use-vendors";
import type { CodeView } from "@/lib/api";

/** The accounts that match the search, under a heading for each app. */
export function usePanelGroups(codes: readonly CodeView[], query: string): VendorGroup[] {
  const vendors = useVendors();
  const matching = codes.filter((view) => matchesSearch(view, query, vendorName(vendors, view.vendor)));
  return groupByVendor(matching, vendors);
}
