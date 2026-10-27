import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ThemeProvider } from "./theme-provider";
import { THEME_STORAGE_KEY } from "./theme-preference";
import { ThemeToggle } from "./theme-toggle";

function stubSystemTheme(prefersDark: boolean) {
  vi.stubGlobal("matchMedia", (query: string) => ({
    matches: prefersDark,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }));
}

beforeEach(() => {
  window.localStorage.clear();
  document.documentElement.className = "";
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const renderToggle = () => render(<ThemeProvider><ThemeToggle /></ThemeProvider>);

describe("ThemeToggle", () => {
  it("defaults to following a dark system", () => {
    stubSystemTheme(true);
    renderToggle();
    expect(screen.getByLabelText("Use system theme").getAttribute("aria-pressed")).toBe("true");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("defaults to light when the system is light", () => {
    stubSystemTheme(false);
    renderToggle();
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });

  it("lets the user override the system and remembers it", () => {
    stubSystemTheme(false);
    renderToggle();
    fireEvent.click(screen.getByLabelText("Use dark theme"));
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
  });
});
