import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { generateRecoveryKey, getVaultFormatVersion, hasRecoveryKey } from "@/lib/api";
import { SecuritySettings } from "./security-settings";

vi.mock("@/lib/api", () => ({
  getVaultFormatVersion: vi.fn(),
  hasRecoveryKey: vi.fn(),
  generateRecoveryKey: vi.fn(),
  assessPassphrase: vi.fn(),
  generatePassphrase: vi.fn(),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function renderSettings(version: number, hasKey = false) {
  vi.mocked(getVaultFormatVersion).mockResolvedValue(version);
  vi.mocked(hasRecoveryKey).mockResolvedValue(hasKey);
  render(<SecuritySettings />);
  await act(async () => {});
}

describe("SecuritySettings", () => {
  it("asks for the upgrade first when the vault is in the older format", async () => {
    await renderSettings(1);
    expect(screen.getByText(/need the newer vault format/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Change passphrase" })).toBeNull();
  });

  it("shows that no recovery key is set and offers to create one", async () => {
    await renderSettings(2, false);
    expect(screen.getByText("Recovery key: not set")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Create recovery key" })).toBeTruthy();
  });

  it("offers to replace an existing recovery key", async () => {
    await renderSettings(2, true);
    expect(screen.getByText("Recovery key: set")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Replace recovery key" })).toBeTruthy();
  });

  it("opens the change passphrase form and the recovery key steps on request", async () => {
    vi.mocked(generateRecoveryKey).mockResolvedValue("a b c");
    await renderSettings(2);
    fireEvent.click(screen.getByRole("button", { name: "Change passphrase" }));
    expect(screen.getByLabelText("Current passphrase")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Create recovery key" })); });
    expect(screen.getByText(/shown only once/)).toBeTruthy();
  });
});
