import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getSettings, setShortcut } from "@/lib/api";
import { PrivacySettingsProvider } from "@/features/privacy/privacy-settings-provider";
import { DEFAULT_SETTINGS } from "@/test-support/settings-fixture";
import { ShortcutSettings } from "./shortcut-settings";

vi.mock("@/lib/api", () => ({ getSettings: vi.fn(), setShortcut: vi.fn() }));

async function renderSettings() {
  vi.mocked(getSettings).mockResolvedValue(DEFAULT_SETTINGS);
  render(<PrivacySettingsProvider><ShortcutSettings /></PrivacySettingsProvider>);
  await act(async () => {});
}

const lockField = () => screen.getByRole("button", { name: /Lock the vault/ });

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("ShortcutSettings", () => {
  it("shows the current shortcuts readably and says when one is not set", async () => {
    await renderSettings();
    expect(screen.getByRole("button", { name: /Show or hide the window/ }).textContent).toBe("Cmd/Ctrl + Shift + A");
    expect(lockField().textContent).toBe("Not set");
  });

  it("saves the keys pressed after clicking the field", async () => {
    await renderSettings();
    vi.mocked(setShortcut).mockResolvedValue({ ...DEFAULT_SETTINGS, shortcuts: { toggle_window: null, lock_vault: "Ctrl+Alt+KeyL" } });
    fireEvent.click(lockField());
    await act(async () => {
      fireEvent.keyDown(lockField(), { code: "KeyL", key: "l", ctrlKey: true, altKey: true });
    });
    expect(setShortcut).toHaveBeenCalledWith("lock_vault", "Ctrl+Alt+KeyL");
    expect(lockField().textContent).toBe("Ctrl + Alt + L");
  });

  it("ignores keys when the field is not listening, and Escape cancels", async () => {
    await renderSettings();
    fireEvent.keyDown(lockField(), { code: "KeyL", ctrlKey: true });
    fireEvent.click(lockField());
    fireEvent.keyDown(lockField(), { code: "Escape", key: "Escape" });
    expect(setShortcut).not.toHaveBeenCalled();
    expect(lockField().textContent).toBe("Not set");
  });

});

describe("ShortcutSettings changes", () => {
  it("clears a shortcut", async () => {
    await renderSettings();
    vi.mocked(setShortcut).mockResolvedValue({ ...DEFAULT_SETTINGS, shortcuts: { toggle_window: null, lock_vault: null } });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    });
    expect(setShortcut).toHaveBeenCalledWith("toggle_window", null);
  });

  it("shows why the system or the rules refused a shortcut", async () => {
    await renderSettings();
    vi.mocked(setShortcut).mockRejectedValue("a shortcut needs a modifier key");
    fireEvent.click(lockField());
    await act(async () => {
      fireEvent.keyDown(lockField(), { code: "KeyL", key: "l" });
    });
    expect(screen.getByRole("alert").textContent).toContain("needs a modifier key");
  });
});
