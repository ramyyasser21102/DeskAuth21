import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getSettings, setCaptureProtection, setClipboardClearSeconds, setHideCodes, setLockOnSleep, setTrayShowsCodes } from "@/lib/api";
import { DEFAULT_SETTINGS } from "@/test-support/settings-fixture";
import { PrivacySettings } from "./privacy-settings";
import { PrivacySettingsProvider } from "./privacy-settings-provider";

vi.mock("@/lib/api", () => ({ getSettings: vi.fn(), setCaptureProtection: vi.fn(), setHideCodes: vi.fn(), setTrayShowsCodes: vi.fn(), setLockOnSleep: vi.fn(), setClipboardClearSeconds: vi.fn() }));

async function renderSettings(settings = DEFAULT_SETTINGS) {
  vi.mocked(getSettings).mockResolvedValue(settings);
  render(<PrivacySettingsProvider><PrivacySettings /></PrivacySettingsProvider>);
  await act(async () => {});
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const captureSwitch = () => screen.getByRole("switch", { name: "Block screen capture" });

describe("PrivacySettings capture protection confirmation", () => {
  it("asks before turning protection off, with the safe choice focused", async () => {
    await renderSettings();
    fireEvent.click(captureSwitch());
    expect(screen.getByText("Turn off screen capture protection?")).toBeTruthy();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Keep protection" }));
    expect(setCaptureProtection).not.toHaveBeenCalled();
  });

  it("keeps protection on when Escape or Keep protection is chosen", async () => {
    await renderSettings();
    fireEvent.click(captureSwitch());
    fireEvent.keyDown(screen.getByRole("button", { name: "Keep protection" }), { key: "Escape" });
    expect(screen.queryByText("Turn off screen capture protection?")).toBeNull();
    expect(setCaptureProtection).not.toHaveBeenCalled();
  });

});

describe("PrivacySettings capture protection changes", () => {
  it("turns protection off only after the confirmation", async () => {
    vi.mocked(setCaptureProtection).mockResolvedValue({ ...DEFAULT_SETTINGS, capture_protection: false });
    await renderSettings();
    fireEvent.click(captureSwitch());
    fireEvent.click(screen.getByRole("button", { name: "Turn off protection" }));
    await waitFor(() => expect(setCaptureProtection).toHaveBeenCalledWith(false));
    await waitFor(() => expect(captureSwitch().getAttribute("aria-checked")).toBe("false"));
  });

  it("turns protection back on without asking", async () => {
    vi.mocked(setCaptureProtection).mockResolvedValue(DEFAULT_SETTINGS);
    await renderSettings({ ...DEFAULT_SETTINGS, capture_protection: false });
    fireEvent.click(captureSwitch());
    await waitFor(() => expect(setCaptureProtection).toHaveBeenCalledWith(true));
  });

  it("is switched off and disabled where the system cannot block capture", async () => {
    await renderSettings({ ...DEFAULT_SETTINGS, capture_protection_supported: false });
    expect(captureSwitch().getAttribute("aria-checked")).toBe("false");
    expect((captureSwitch() as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(/no way to block screen capture/)).toBeTruthy();
  });
});

describe("PrivacySettings other choices", () => {
  it("saves hidden codes and tray codes", async () => {
    vi.mocked(setHideCodes).mockResolvedValue({ ...DEFAULT_SETTINGS, hide_codes: false });
    vi.mocked(setTrayShowsCodes).mockResolvedValue({ ...DEFAULT_SETTINGS, tray_shows_codes: true });
    await renderSettings({ ...DEFAULT_SETTINGS, tray_menu_lists_accounts: true });
    fireEvent.click(screen.getByRole("switch", { name: "Hide codes until revealed" }));
    fireEvent.click(screen.getByRole("switch", { name: "Show codes in the tray menu" }));
    await waitFor(() => expect(setHideCodes).toHaveBeenCalledWith(false));
    await waitFor(() => expect(setTrayShowsCodes).toHaveBeenCalledWith(true));
  });

  it("shows the message when saving fails", async () => {
    vi.mocked(setHideCodes).mockRejectedValue("could not save settings");
    await renderSettings();
    fireEvent.click(screen.getByRole("switch", { name: "Hide codes until revealed" }));
    expect(await screen.findByText("could not save settings")).toBeTruthy();
  });
});

describe("PrivacySettings locking and clipboard", () => {
  it("shows lock on sleep as on by default and saves turning it off", async () => {
    vi.mocked(setLockOnSleep).mockResolvedValue({ ...DEFAULT_SETTINGS, lock_on_sleep: false });
    await renderSettings();
    const lockSwitch = screen.getByRole("switch", { name: "Lock when the computer sleeps or the screen locks" });
    expect(lockSwitch.getAttribute("aria-checked")).toBe("true");
    fireEvent.click(lockSwitch);
    await waitFor(() => expect(setLockOnSleep).toHaveBeenCalledWith(false));
  });

  it("marks the saved clipboard time and saves a new choice", async () => {
    vi.mocked(setClipboardClearSeconds).mockResolvedValue({ ...DEFAULT_SETTINGS, clipboard_clear_seconds: 60 });
    await renderSettings();
    expect(screen.getByRole("button", { name: "15 s" }).getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(screen.getByRole("button", { name: "60 s" }));
    await waitFor(() => expect(setClipboardClearSeconds).toHaveBeenCalledWith(60));
    expect(await screen.findByText(/after 60 seconds/)).toBeTruthy();
  });
});

describe("PrivacySettings tray switch", () => {
  it("is offered only where the tray menu lists the accounts", async () => {
    await renderSettings();
    expect(screen.queryByRole("switch", { name: "Show codes in the tray menu" })).toBeNull();
    cleanup();
    await renderSettings({ ...DEFAULT_SETTINGS, tray_menu_lists_accounts: true });
    expect(screen.getByRole("switch", { name: "Show codes in the tray menu" })).toBeTruthy();
  });
});
