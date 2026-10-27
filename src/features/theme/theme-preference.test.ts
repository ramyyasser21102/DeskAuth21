import { describe, expect, it } from "vitest";
import {
  applyTheme,
  DEFAULT_THEME_PREFERENCE,
  readStoredPreference,
  resolveIsDark,
  storePreference,
  THEME_STORAGE_KEY,
} from "./theme-preference";

const storageHolding = (value: string | null) => ({ getItem: () => value });

describe("resolveIsDark", () => {
  it("follows the system only when the preference is system", () => {
    expect(resolveIsDark("system", true)).toBe(true);
    expect(resolveIsDark("system", false)).toBe(false);
    expect(resolveIsDark("light", true)).toBe(false);
    expect(resolveIsDark("dark", false)).toBe(true);
  });
});

describe("readStoredPreference", () => {
  it("defaults to system when nothing or junk is stored", () => {
    expect(readStoredPreference(storageHolding(null))).toBe(DEFAULT_THEME_PREFERENCE);
    expect(readStoredPreference(storageHolding("purple"))).toBe("system");
  });

  it("returns a valid stored choice", () => {
    expect(readStoredPreference(storageHolding("dark"))).toBe("dark");
  });

  it("falls back when storage throws", () => {
    const blocked = { getItem: () => { throw new Error("blocked"); } };
    expect(readStoredPreference(blocked)).toBe("system");
  });
});

describe("storePreference", () => {
  it("saves under the app key and tolerates blocked storage", () => {
    const saved: Record<string, string> = {};
    storePreference({ setItem: (key, value) => { saved[key] = value; } }, "light");
    expect(saved[THEME_STORAGE_KEY]).toBe("light");
    const blocked = { setItem: () => { throw new Error("blocked"); } };
    expect(() => storePreference(blocked, "dark")).not.toThrow();
  });
});

describe("applyTheme", () => {
  it("toggles the dark class and colour scheme", () => {
    const root = document.createElement("html");
    applyTheme("dark", root, false);
    expect(root.classList.contains("dark")).toBe(true);
    expect(root.style.colorScheme).toBe("dark");
    applyTheme("system", root, false);
    expect(root.classList.contains("dark")).toBe(false);
  });
});
