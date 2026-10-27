import { cn } from "@/lib/utils";
import type { VendorCount, VendorFilter } from "./vendor-filter";
import { vendorName, useVendors } from "./use-vendors";

/** With a single app there is nothing to switch between. */
const MIN_VENDORS_TO_SHOW_TABS = 2;

interface VendorTabsProps {
  counts: readonly VendorCount[];
  total: number;
  selected: VendorFilter;
  onSelect: (filter: VendorFilter) => void;
}

const chipClass = (isSelected: boolean) =>
  cn(
    "rounded-full border px-3 py-1.5 text-sm font-medium transition-colors",
    isSelected ? "border-primary bg-primary text-primary-foreground" : "hover:bg-accent",
  );

/** One chip per app that has accounts, so users can flip between sources without leaving the page. */
export function VendorTabs({ counts, total, selected, onSelect }: VendorTabsProps) {
  const vendors = useVendors();
  if (counts.length < MIN_VENDORS_TO_SHOW_TABS) return null;
  return (
    <div role="group" aria-label="Filter by app" className="flex flex-wrap gap-2">
      <button type="button" aria-pressed={selected === "all"} className={chipClass(selected === "all")} onClick={() => onSelect("all")}>
        All ({total})
      </button>
      {counts.map(({ vendor, count }) => (
        <button
          key={vendor}
          type="button"
          aria-pressed={selected === vendor}
          className={chipClass(selected === vendor)}
          onClick={() => onSelect(vendor)}
        >
          {vendorName(vendors, vendor)} ({count})
        </button>
      ))}
    </div>
  );
}
