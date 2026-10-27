export type ThemePreference = "system" | "light" | "dark";

export const THEME_STORAGE_KEY = "deskauth21.theme";
export const DEFAULT_THEME_PREFERENCE: ThemePreference = "system";
export const DARK_MODE_QUERY = "(prefers-color-scheme: dark)";

const VALID_PREFERENCES: readonly string[] = ["system", "light", "dark"];

export function isThemePreference(value: unknown): value is ThemePreference {
  return typeof value === "string" && VALID_PREFERENCES.includes(value);
}

export function resolveIsDark(preference: ThemePreference, systemPrefersDark: boolean): boolean {
  return preference === "system" ? systemPrefersDark : preference === "dark";
}

/** Reads the saved choice; storage can be blocked or hold junk, so fall back to the default. */
export function readStoredPreference(storage: Pick<Storage, "getItem">): ThemePreference {
  try {
    const stored = storage.getItem(THEME_STORAGE_KEY);
    return isThemePreference(stored) ? stored : DEFAULT_THEME_PREFERENCE;
  } catch {
    return DEFAULT_THEME_PREFERENCE;
  }
}

export function storePreference(storage: Pick<Storage, "setItem">, preference: ThemePreference): void {
  try {
    storage.setItem(THEME_STORAGE_KEY, preference);
  } catch {
    // Persisting is a convenience; the choice still applies for this session.
  }
}

export function applyTheme(preference: ThemePreference, root: HTMLElement, systemPrefersDark: boolean): void {
  const isDark = resolveIsDark(preference, systemPrefersDark);
  root.classList.toggle("dark", isDark);
  root.style.colorScheme = isDark ? "dark" : "light";
}

/** Accessed lazily so a blocked `localStorage` throws inside the guarded readers/writers. */
export const browserStorage = {
  getItem: (key: string) => window.localStorage.getItem(key),
  setItem: (key: string, value: string) => window.localStorage.setItem(key, value),
};

/** Called before the first render so the page never flashes the wrong theme. */
export function applyStoredTheme(): void {
  const systemPrefersDark = window.matchMedia(DARK_MODE_QUERY).matches;
  applyTheme(readStoredPreference(browserStorage), document.documentElement, systemPrefersDark);
}
