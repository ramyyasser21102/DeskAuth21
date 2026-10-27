import { useEffect } from "react";

/** Calls `onImages` when the user pastes image files (e.g. a screenshot) anywhere on the page. */
export function usePasteImages(onImages: (files: File[]) => void): void {
  useEffect(() => {
    const handlePaste = (event: ClipboardEvent) => {
      const files = Array.from(event.clipboardData?.files ?? []).filter((file) => file.type.startsWith("image/"));
      if (files.length === 0) return;
      event.preventDefault();
      onImages(files);
    };
    document.addEventListener("paste", handlePaste);
    return () => document.removeEventListener("paste", handlePaste);
  }, [onImages]);
}
