import { motion } from "motion/react";
import { ENTER } from "@/lib/motion-tokens";
import { cn } from "@/lib/utils";
import { PAGE_TABS, type PageId } from "./pages";

interface TabBarProps {
  current: PageId;
  onSelect: (page: PageId) => void;
}

/** Fixed to the bottom edge so the three pages are always one tap away. */
export function TabBar({ current, onSelect }: TabBarProps) {
  return (
    <nav aria-label="Pages" className="fixed inset-x-0 bottom-0 border-t bg-background">
      <ul className="mx-auto flex max-w-md">
        {PAGE_TABS.map(({ id, label, Icon }) => (
          <li key={id} className="relative flex-1">
            {current === id && <motion.span layoutId="tab-indicator" aria-hidden className="absolute inset-x-6 top-0 h-0.5 bg-foreground" transition={ENTER} />}
            <button
              type="button"
              aria-current={current === id ? "page" : undefined}
              onClick={() => onSelect(id)}
              className={cn(
                "group flex w-full flex-col items-center gap-1 py-2.5 text-xs font-medium transition-colors hover:text-foreground",
                current === id ? "text-foreground" : "text-muted-foreground",
              )}
            >
              <Icon className="size-5 transition-transform group-hover:-translate-y-0.5" />
              {label}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}
