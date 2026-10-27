import { useEffect, useState } from "react";
import { listCodes, type CodeView } from "@/lib/api";

const REFRESH_INTERVAL_MS = 1000;
export type PanelStatus = "loading" | "ready" | "locked";

/** The accounts and their live codes, asked for every second; "locked" when they cannot be read. */
export function usePanelCodes(): { codes: CodeView[]; status: PanelStatus } {
  const [codes, setCodes] = useState<CodeView[]>([]);
  const [status, setStatus] = useState<PanelStatus>("loading");

  useEffect(() => {
    let isActive = true;
    const refresh = async () => {
      try {
        const latest = await listCodes();
        if (isActive) {
          setCodes(latest);
          setStatus("ready");
        }
      } catch {
        if (isActive) {
          setCodes([]);
          setStatus("locked");
        }
      }
    };
    void refresh();
    const timerId = window.setInterval(refresh, REFRESH_INTERVAL_MS);
    return () => {
      isActive = false;
      window.clearInterval(timerId);
    };
  }, []);

  return { codes, status };
}
