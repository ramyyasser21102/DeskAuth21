import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getSettings, setAutoLockMinutes } from "@/lib/api";
import { DEFAULT_SETTINGS } from "@/test-support/settings-fixture";
import { AutoLockSetting } from "./auto-lock-setting";

vi.mock("@/lib/api", () => ({ getSettings: vi.fn(), setAutoLockMinutes: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const isPressed = (name: string) => screen.getByRole("button", { name }).getAttribute("aria-pressed");

describe("AutoLockSetting display", () => {
  it("shows the saved choice and what it means", async () => {
    vi.mocked(getSettings).mockResolvedValue({ ...DEFAULT_SETTINGS, auto_lock_minutes: 5 });
    render(<AutoLockSetting />);
    expect(await screen.findByText("The vault locks after 5 minutes without activity.")).toBeTruthy();
    expect(isPressed("5 min")).toBe("true");
    expect(isPressed("Never")).toBe("false");
  });

  it("renders nothing until the setting has loaded", () => {
    vi.mocked(getSettings).mockReturnValue(new Promise(() => {}));
    const { container } = render(<AutoLockSetting />);
    expect(container.innerHTML).toBe("");
  });
});

describe("AutoLockSetting saving", () => {
  it("saves a new choice through the backend and reflects it", async () => {
    vi.mocked(getSettings).mockResolvedValue({ ...DEFAULT_SETTINGS, auto_lock_minutes: 5 });
    vi.mocked(setAutoLockMinutes).mockResolvedValue({ ...DEFAULT_SETTINGS, auto_lock_minutes: null });
    render(<AutoLockSetting />);
    fireEvent.click(await screen.findByRole("button", { name: "Never" }));
    await waitFor(() => expect(isPressed("Never")).toBe("true"));
    expect(setAutoLockMinutes).toHaveBeenCalledWith(null);
    expect(screen.getByText("The vault stays unlocked until you lock it.")).toBeTruthy();
  });

  it("keeps the old choice and shows the error when saving fails", async () => {
    vi.mocked(getSettings).mockResolvedValue({ ...DEFAULT_SETTINGS, auto_lock_minutes: 5 });
    vi.mocked(setAutoLockMinutes).mockRejectedValue("could not access the vault file");
    render(<AutoLockSetting />);
    fireEvent.click(await screen.findByRole("button", { name: "30 min" }));
    expect(await screen.findByText("could not access the vault file")).toBeTruthy();
    expect(isPressed("5 min")).toBe("true");
  });
});
