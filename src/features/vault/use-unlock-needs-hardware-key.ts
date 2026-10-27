import { useEffect, useState } from "react";
import { unlockNeedsHardwareKey } from "@/lib/api";

/** Whether this vault needs a hardware key at unlock; false until known, and if it cannot be read. */
export function useUnlockNeedsHardwareKey(): boolean {
  const [needsKey, setNeedsKey] = useState(false);
  useEffect(() => {
    let isActive = true;
    unlockNeedsHardwareKey()
      .then((needed) => isActive && setNeedsKey(needed))
      .catch(() => isActive && setNeedsKey(false));
    return () => {
      isActive = false;
    };
  }, []);
  return needsKey;
}
