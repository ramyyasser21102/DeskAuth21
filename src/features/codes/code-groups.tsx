import { AnimatePresence } from "motion/react";
import { Button } from "@/components/ui/button";
import type { VendorGroup as Group } from "@/features/vendors/search-and-group";
import { VendorGroup } from "./vendor-group";

interface CodeGroupsProps {
  groups: readonly Group[];
  query: string;
  onClearSearch: () => void;
}

/** The grouped list, or a clear way back when the search matches nothing. */
export function CodeGroups({ groups, query, onClearSearch }: CodeGroupsProps) {
  if (groups.length === 0 && query.trim() !== "") {
    return (
      <div className="flex flex-col items-start gap-2 py-2">
        <p className="text-sm">{`No accounts match "${query.trim()}".`}</p>
        <Button type="button" variant="outline" size="sm" onClick={onClearSearch}>Show all accounts</Button>
      </div>
    );
  }
  return <AnimatePresence initial={false}>{groups.map((group) => <VendorGroup key={group.vendor} group={group} />)}</AnimatePresence>;
}
