import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { getSettings, type Settings } from "@/lib/api";

interface PrivacySettingsValue {
  /** `null` until the saved settings have loaded. */
  settings: Settings | null;
  /** Takes the settings a command returned, so every screen sees the change at once. */
  applySaved: (saved: Settings) => void;
}

const PrivacySettingsContext = createContext<PrivacySettingsValue | null>(null);

export function PrivacySettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  useEffect(() => {
    let isActive = true;
    void getSettings().then((loaded) => isActive && setSettings(loaded));
    return () => {
      isActive = false;
    };
  }, []);
  const applySaved = useCallback((saved: Settings) => setSettings(saved), []);
  const value = useMemo(() => ({ settings, applySaved }), [settings, applySaved]);
  return <PrivacySettingsContext.Provider value={value}>{children}</PrivacySettingsContext.Provider>;
}

export function usePrivacySettings(): PrivacySettingsValue {
  const context = useContext(PrivacySettingsContext);
  if (!context) throw new Error("usePrivacySettings must be used inside <PrivacySettingsProvider>");
  return context;
}
