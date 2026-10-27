import { useCallback, useRef, useState } from "react";
import { copyCode, hideQuickPanel } from "@/lib/api";
import { PanelFooter } from "./panel-footer";
import { PanelList } from "./panel-list";
import { PanelLocked } from "./panel-locked";
import { PanelSearch } from "./panel-search";
import { usePanelCodes } from "./use-panel-codes";
import { usePanelFocus } from "./use-panel-focus";
import { usePanelGroups } from "./use-panel-groups";
import { usePanelKeys } from "./use-panel-keys";

const close = () => void hideQuickPanel();

/** The page the menu bar icon opens: search, every account by app, and the way back to the app. */
export function QuickPanel() {
  const { codes, status } = usePanelCodes();
  const [query, setQuery] = useState("");
  const searchRef = useRef<HTMLInputElement>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const groups = usePanelGroups(codes, query);
  const clearQuery = useCallback(() => setQuery(""), []);
  usePanelFocus(searchRef, clearQuery);
  const handleKeyDown = usePanelKeys({ query, onClearQuery: clearQuery, rootRef, searchRef });

  const copyFirstMatch = async () => {
    const first = groups[0]?.views[0];
    if (first) await copyCode(first.id).then(close).catch(() => {});
  };

  return (
    <div ref={rootRef} onKeyDown={handleKeyDown} className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      {status === "locked" ? <PanelLocked /> : (
        <>
          <PanelSearch query={query} onChange={setQuery} onEnter={() => void copyFirstMatch()} inputRef={searchRef} />
          <PanelList groups={groups} hasAccounts={codes.length > 0} query={query} onCopied={close} />
        </>
      )}
      <PanelFooter />
    </div>
  );
}
