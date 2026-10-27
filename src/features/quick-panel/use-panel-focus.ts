import { useEffect, type RefObject } from "react";

/**
 * The panel's page stays alive while it is hidden. When it is shown, the search box gets the
 * cursor so typing starts at once; when it is left, the search is cleared for next time.
 */
export function usePanelFocus(searchRef: RefObject<HTMLInputElement | null>, onLeave: () => void) {
  useEffect(() => {
    const onFocus = () => {
      searchRef.current?.focus();
      searchRef.current?.select();
    };
    window.addEventListener("focus", onFocus);
    window.addEventListener("blur", onLeave);
    onFocus();
    return () => {
      window.removeEventListener("focus", onFocus);
      window.removeEventListener("blur", onLeave);
    };
  }, [searchRef, onLeave]);
}
