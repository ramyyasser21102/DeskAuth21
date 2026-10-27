import { useCallback, useEffect, useState } from "react";
import { getSettings, setAutoLockMinutes } from "@/lib/api";

/** `minutes` is `undefined` until the saved setting has loaded. */
export function useAutoLockSetting() {
  const [minutes, setMinutes] = useState<number | null | undefined>(undefined);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    let isActive = true;
    void getSettings().then((settings) => {
      if (isActive) setMinutes(settings.auto_lock_minutes);
    });
    return () => {
      isActive = false;
    };
  }, []);

  const choose = useCallback(async (next: number | null) => {
    try {
      const saved = await setAutoLockMinutes(next);
      setMinutes(saved.auto_lock_minutes);
      setErrorMessage(null);
    } catch (error) {
      setErrorMessage(String(error));
    }
  }, []);

  return { minutes, errorMessage, choose };
}
