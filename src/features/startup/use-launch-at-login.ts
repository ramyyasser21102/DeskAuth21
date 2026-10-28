import { useEffect, useState } from "react";
import { getLaunchAtLogin, setLaunchAtLogin } from "@/lib/api";

/** The system's launch-at-login setting, changed only when the system confirms it. */
export function useLaunchAtLogin() {
  const [isOn, setIsOn] = useState<boolean | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    let isActive = true;
    void getLaunchAtLogin().then((current) => isActive && setIsOn(current));
    return () => {
      isActive = false;
    };
  }, []);

  const choose = async (wanted: boolean) => {
    try {
      setIsOn(await setLaunchAtLogin(wanted));
      setErrorMessage(null);
    } catch (error) {
      setErrorMessage(String(error));
      setIsOn(await getLaunchAtLogin());
    }
  };

  return { isOn, errorMessage, choose };
}
