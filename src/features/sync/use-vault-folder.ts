import { useCallback, useEffect, useState } from "react";
import { chooseVaultFolder, getVaultFolder } from "@/lib/api";
import { describeRelocation, requiresUnlockAgain } from "./relocation-message";

export interface FolderMessage {
  text: string;
  isError: boolean;
}

/** Shows the vault's folder and lets the user pick another (for example a synced one). */
export function useVaultFolder(onVaultChanged: () => void) {
  const [folder, setFolder] = useState<string | null>(null);
  const [message, setMessage] = useState<FolderMessage | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  useEffect(() => {
    let isActive = true;
    void getVaultFolder().then((current) => {
      if (isActive) setFolder(current);
    });
    return () => {
      isActive = false;
    };
  }, []);

  const choose = useCallback(async () => {
    setIsBusy(true);
    try {
      const outcome = await chooseVaultFolder();
      if (outcome === null) return;
      setMessage({ text: describeRelocation(outcome), isError: false });
      setFolder(await getVaultFolder());
      if (requiresUnlockAgain(outcome)) onVaultChanged();
    } catch (error) {
      setMessage({ text: String(error), isError: true });
    } finally {
      setIsBusy(false);
    }
  }, [onVaultChanged]);

  return { folder, message, isBusy, choose };
}
