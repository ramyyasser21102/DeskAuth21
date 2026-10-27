import { useCallback, useEffect, useState } from "react";
import { getHardwareKeyStatus, type HardwareKeyStatus } from "@/lib/api";

/** Which hardware keys the open vault needs, and a way to ask again after a change. */
export function useHardwareKeys() {
  const [status, setStatus] = useState<HardwareKeyStatus | null>(null);
  const refresh = useCallback(() => {
    getHardwareKeyStatus().then(setStatus).catch(() => setStatus(null));
  }, []);
  useEffect(refresh, [refresh]);
  return { status, refresh };
}
