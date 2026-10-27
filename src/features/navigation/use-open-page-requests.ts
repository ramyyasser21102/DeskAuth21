import { useEffect } from "react";
import { isPageId, type PageId } from "./pages";

const OPEN_PAGE_EVENT = "open-page";

/**
 * The tray menu asks the main window to show a page (the settings, say) by sending an event.
 * Outside the real app there is nothing to listen to, so this does nothing.
 */
export function useOpenPageRequests(onPage: (page: PageId) => void) {
  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    let isActive = true;
    let stopListening: (() => void) | undefined;
    import("@tauri-apps/api/event")
      .then(({ listen }) => listen<unknown>(OPEN_PAGE_EVENT, ({ payload }) => isPageId(payload) && onPage(payload)))
      .then((stop) => (isActive ? (stopListening = stop) : stop()))
      .catch((error) => console.error("could not listen for page requests:", error));
    return () => {
      isActive = false;
      stopListening?.();
    };
  }, [onPage]);
}
