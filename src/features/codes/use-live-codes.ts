import { useEffect, useState } from "react";
import { listCodes, type CodeView } from "@/lib/api";

const REFRESH_INTERVAL_MS = 1000;

export interface LiveCodes {
  /** `null` until the first answer arrives. */
  codes: CodeView[] | null;
  /** Whether the very first answer was empty; fixed afterwards. `null` until then. */
  hadNoAccountsOnLoad: boolean | null;
}

/**
 * Polls the current codes every second. `onUnavailable` fires when they can no
 * longer be read (usually because the vault was locked from the tray).
 */
export function useLiveCodes(onUnavailable: () => void): LiveCodes {
  const [codes, setCodes] = useState<CodeView[] | null>(null);
  const [hadNoAccountsOnLoad, setHadNoAccountsOnLoad] = useState<boolean | null>(null);

  useEffect(() => {
    let isActive = true;
    const refresh = async () => {
      try {
        const latest = await listCodes();
        if (!isActive) return;
        setCodes(latest);
        setHadNoAccountsOnLoad((previous) => previous ?? latest.length === 0);
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

  return { codes, hadNoAccountsOnLoad };
}
