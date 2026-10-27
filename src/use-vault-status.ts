import { useCallback, useEffect, useState } from "react";
import { getVaultStatus, type VaultStatus } from "@/lib/api";

/** `status` is null until the first answer from the backend arrives. */
export function useVaultStatus() {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const refreshStatus = useCallback(async () => setStatus(await getVaultStatus()), []);

  useEffect(() => {
    let isActive = true;
    void getVaultStatus().then((initialStatus) => {
      if (isActive) setStatus(initialStatus);
    });
    return () => {
      isActive = false;
    };
  }, []);

  return { status, refreshStatus };
}
