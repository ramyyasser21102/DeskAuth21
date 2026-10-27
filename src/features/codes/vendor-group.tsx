import { AnimatePresence, motion } from "motion/react";
import { ROW_MOTION } from "@/lib/motion-tokens";
import type { VendorGroup as Group } from "@/features/vendors/search-and-group";
import { CodeRow } from "./code-row";

/** One app's accounts under a heading that says how many there are. */
export function VendorGroup({ group }: { group: Group }) {
  const headingId = `group-${group.vendor}`;
  return (
    <motion.section layout="position" aria-labelledby={headingId} className="flex flex-col" {...ROW_MOTION}>
      <h3 id={headingId} className="px-2 pt-3 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        {group.name} ({group.views.length})
      </h3>
      <ul>
        <AnimatePresence initial={false}>
          {group.views.map((view) => (
            <CodeRow key={view.id} view={view} />
          ))}
        </AnimatePresence>
      </ul>
    </motion.section>
  );
}
