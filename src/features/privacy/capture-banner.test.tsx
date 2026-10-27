import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getSettings, setCaptureProtection, setHideCodes } from "@/lib/api";
import { DEFAULT_SETTINGS } from "@/test-support/settings-fixture";
import { CaptureBanner } from "./capture-banner";
import { PrivacySettingsProvider } from "./privacy-settings-provider";

vi.mock("@/lib/api", () => ({ getSettings: vi.fn(), setCaptureProtection: vi.fn(), setHideCodes: vi.fn() }));

async function renderBanner(settings = DEFAULT_SETTINGS) {
  vi.mocked(getSettings).mockResolvedValue(settings);
  render(<PrivacySettingsProvider><CaptureBanner /></PrivacySettingsProvider>);
  await act(async () => {});
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("CaptureBanner", () => {
  it("is absent while capture protection is working", async () => {
    await renderBanner();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("warns that codes can be captured when protection is off, and turns it back on", async () => {
    await renderBanner({ ...DEFAULT_SETTINGS, capture_protection: false });
    expect(screen.getByRole("alert").textContent).toContain("can capture your codes");
    vi.mocked(setCaptureProtection).mockResolvedValue(DEFAULT_SETTINGS);
    fireEvent.click(screen.getByRole("button", { name: "Turn on protection" }));
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
    expect(setCaptureProtection).toHaveBeenCalledWith(true);
  });

  it("explains an unsupported system and offers to hide codes when they are visible", async () => {
    const unsupported = { ...DEFAULT_SETTINGS, capture_protection_supported: false, hide_codes: false };
    await renderBanner(unsupported);
    expect(screen.getByRole("alert").textContent).toContain("cannot block screen capture");
    vi.mocked(setHideCodes).mockResolvedValue({ ...unsupported, hide_codes: true });
    fireEvent.click(screen.getByRole("button", { name: "Hide codes" }));
    await waitFor(() => expect(setHideCodes).toHaveBeenCalledWith(true));
  });

  it("stays, without a button, when the system cannot block capture but codes are hidden", async () => {
    await renderBanner({ ...DEFAULT_SETTINGS, capture_protection_supported: false });
    expect(screen.getByRole("alert")).toBeTruthy();
    expect(screen.queryByRole("button")).toBeNull();
  });
});
