import { useEffect, useState } from "react";
import { listCodes, type CodeView } from "@/lib/api";

const REFRESH_INTERVAL_MS = 1000;

/**
 * Polls the current codes every second. `onUnavailable` fires when they can no
 * longer be read (usually because the vault was locked from the tray).
 */
export function useLiveCodes(onUnavailable: () => void): CodeView[] {
  const [codes, setCodes] = useState<CodeView[]>([]);

  useEffect(() => {
    let isActive = true;
    const refresh = async () => {
      try {
        const latest = await listCodes();
        if (isActive) setCodes(latest);
      } catch {
        if (isActive) onUnavailable();
      }
    };
    void refresh();
    const timerId = window.setInterval(refresh, REFRESH_INTERVAL_MS);
    return () => {
      isActive = false;
      window.clearInterval(timerId);
    };
  }, [onUnavailable]);

  return codes;
}
