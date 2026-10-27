import { useState } from "react";
import { removeHardwareKey } from "@/lib/api";

/** Which key is waiting to be confirmed for removal, and the removal itself. */
export function useRemoveHardwareKey(onChanged: () => void) {
  const [pendingId, setPendingId] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const remove = async (id: string) => {
    try {
      await removeHardwareKey(id);
      setPendingId(null);
      onChanged();
    } catch (error) {
      setErrorMessage(String(error));
    }
  };

  return { pendingId, askToRemove: setPendingId, errorMessage, remove };
}
