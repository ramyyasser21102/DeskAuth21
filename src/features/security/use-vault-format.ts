import { useCallback, useEffect, useState } from "react";
import { getVaultFormatVersion, upgradeVaultFormat } from "@/lib/api";

export const CURRENT_FORMAT_VERSION = 2;

/** Which file format the open vault is in, and a way to move an older one to the current format. */
export function useVaultFormat() {
  const [version, setVersion] = useState<number | null>(null);
  const [isUpgrading, setIsUpgrading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    let isActive = true;
    getVersionOrNull().then((found) => isActive && setVersion(found));
    return () => {
      isActive = false;
    };
  }, []);

  const upgrade = useCallback(async () => {
    setIsUpgrading(true);
    setErrorMessage(null);
    try {
      await upgradeVaultFormat();
      setVersion(await getVaultFormatVersion());
    } catch (error) {
      setErrorMessage(String(error));
    } finally {
      setIsUpgrading(false);
    }
  }, []);

  return { version, isUpgrading, errorMessage, upgrade };
}

/** Asking can fail if the vault locks meanwhile; then there is simply nothing to offer. */
async function getVersionOrNull(): Promise<number | null> {
  try {
    return await getVaultFormatVersion();
  } catch {
    return null;
  }
}
