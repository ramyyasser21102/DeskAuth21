import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getVaultFormatVersion, upgradeVaultFormat } from "@/lib/api";
import { UpgradePrompt } from "./upgrade-prompt";

vi.mock("@/lib/api", () => ({ getVaultFormatVersion: vi.fn(), upgradeVaultFormat: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function renderPrompt(version: number) {
  vi.mocked(getVaultFormatVersion).mockResolvedValue(version);
  render(<UpgradePrompt />);
  await act(async () => {});
}

describe("UpgradePrompt offer", () => {
  it("says nothing for a vault already in the current format", async () => {
    await renderPrompt(2);
    expect(screen.queryByText(/Upgrade your vault/)).toBeNull();
  });

  it("explains the upgrade, the safety copy and the other computers for an older vault", async () => {
    await renderPrompt(1);
    expect(screen.getByText("Upgrade your vault's security format")).toBeTruthy();
    expect(screen.getByText(/Computers that share this vault must update/)).toBeTruthy();
    expect(upgradeVaultFormat).not.toHaveBeenCalled();
  });

  it("only hides itself for now when Not now is chosen", async () => {
    await renderPrompt(1);
    fireEvent.click(screen.getByRole("button", { name: "Not now" }));
    expect(screen.queryByText(/Upgrade your vault/)).toBeNull();
    expect(upgradeVaultFormat).not.toHaveBeenCalled();
  });

});

describe("UpgradePrompt upgrading", () => {
  it("upgrades and then confirms where the old file went", async () => {
    vi.mocked(upgradeVaultFormat).mockResolvedValue(undefined);
    await renderPrompt(1);
    vi.mocked(getVaultFormatVersion).mockResolvedValue(2);
    fireEvent.click(screen.getByRole("button", { name: "Upgrade vault" }));
    expect(await screen.findByText(/Vault upgraded/)).toBeTruthy();
    expect(upgradeVaultFormat).toHaveBeenCalledOnce();
  });

  it("shows why it failed and stays available", async () => {
    vi.mocked(upgradeVaultFormat).mockRejectedValue("the vault file was replaced by a different vault");
    await renderPrompt(1);
    fireEvent.click(screen.getByRole("button", { name: "Upgrade vault" }));
    expect(await screen.findByText("the vault file was replaced by a different vault")).toBeTruthy();
    await waitFor(() => expect((screen.getByRole("button", { name: "Upgrade vault" }) as HTMLButtonElement).disabled).toBe(false));
  });
});
