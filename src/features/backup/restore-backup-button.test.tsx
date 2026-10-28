import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { restoreVaultBackup } from "@/lib/api";
import { RestoreBackupButton } from "./restore-backup-button";

vi.mock("@/lib/api", () => ({ restoreVaultBackup: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const click = async (name: string) => act(async () => { fireEvent.click(screen.getByRole("button", { name })); });

describe("RestoreBackupButton", () => {
  it("restores at once when there is no vault to replace", async () => {
    vi.mocked(restoreVaultBackup).mockResolvedValue(true);
    const onRestored = vi.fn();
    render(<RestoreBackupButton replacesVault={false} onRestored={onRestored} />);
    await click("Restore from a backup file");
    expect(onRestored).toHaveBeenCalledOnce();
  });

  it("asks first when it would replace a vault, and Cancel changes nothing", async () => {
    render(<RestoreBackupButton replacesVault onRestored={() => {}} />);
    await click("Restore from a backup file");
    expect(screen.getByText(/replaces the vault on this computer/)).toBeTruthy();
    await click("Cancel");
    expect(restoreVaultBackup).not.toHaveBeenCalled();
  });

  it("does nothing when the file dialog is cancelled, and shows a refusal", async () => {
    const onRestored = vi.fn();
    vi.mocked(restoreVaultBackup).mockResolvedValueOnce(false).mockRejectedValueOnce("not a DA21 vault");
    render(<RestoreBackupButton replacesVault={false} onRestored={onRestored} />);
    await click("Restore from a backup file");
    await click("Restore from a backup file");
    expect(onRestored).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toContain("not a DA21 vault");
  });
});
