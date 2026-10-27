import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import {
  applyTheme,
  browserStorage,
  DARK_MODE_QUERY,
  readStoredPreference,
  storePreference,
  type ThemePreference,
} from "./theme-preference";

interface ThemeContextValue {
  preference: ThemePreference;
  setPreference: (preference: ThemePreference) => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

/** Keeps the page in step with the preference and, for "system", with OS changes. */
function useThemeSync(preference: ThemePreference): void {
  useEffect(() => {
    const systemQuery = window.matchMedia(DARK_MODE_QUERY);
    const apply = () => applyTheme(preference, document.documentElement, systemQuery.matches);
    apply();
    systemQuery.addEventListener("change", apply);
    return () => systemQuery.removeEventListener("change", apply);
  }, [preference]);
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [preference, setPreferenceState] = useState(() => readStoredPreference(browserStorage));
  useThemeSync(preference);

  const setPreference = useCallback((next: ThemePreference) => {
    storePreference(browserStorage, next);
    setPreferenceState(next);
  }, []);
  const value = useMemo(() => ({ preference, setPreference }), [preference, setPreference]);

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used inside <ThemeProvider>");
  return context;
}
