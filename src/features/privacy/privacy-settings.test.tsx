import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getSettings, setCaptureProtection, setHideCodes, setTrayShowsCodes } from "@/lib/api";
import { DEFAULT_SETTINGS } from "@/test-support/settings-fixture";
import { PrivacySettings } from "./privacy-settings";
import { PrivacySettingsProvider } from "./privacy-settings-provider";

vi.mock("@/lib/api", () => ({ getSettings: vi.fn(), setCaptureProtection: vi.fn(), setHideCodes: vi.fn(), setTrayShowsCodes: vi.fn() }));

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
    await renderSettings();
    fireEvent.click(screen.getByRole("switch", { name: "Hide codes until revealed" }));
    fireEvent.click(screen.getByRole("switch", { name: "Show codes in the menu bar menu" }));
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
