import type { KeyboardEvent, RefObject } from "react";
import { hideQuickPanel } from "@/lib/api";
import { nextRowIndex } from "./panel-keys";

const ROW_SELECTOR = 'button[aria-label^="Copy code for"]';

interface PanelKeysOptions {
  query: string;
  onClearQuery: () => void;
  rootRef: RefObject<HTMLElement | null>;
  searchRef: RefObject<HTMLInputElement | null>;
}

/** Escape clears the search, then closes the panel; the arrow keys walk between the search box and the rows. */
export function usePanelKeys({ query, onClearQuery, rootRef, searchRef }: PanelKeysOptions) {
  const moveFocus = (key: "ArrowDown" | "ArrowUp") => {
    const rows = Array.from(rootRef.current?.querySelectorAll<HTMLElement>(ROW_SELECTOR) ?? []);
    const target = nextRowIndex(rows.indexOf(document.activeElement as HTMLElement), key, rows.length);
    (target === -1 ? searchRef.current : rows[target])?.focus();
  };

  return (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      if (query === "") void hideQuickPanel();
      else onClearQuery();
    } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      moveFocus(event.key);
    }
  };
}
