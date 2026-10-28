import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { exportAccountsFile, restoreVaultBackup, saveVaultBackup } from "@/lib/api";
import { BackupSettings } from "./backup-settings";

vi.mock("@/lib/api", () => ({ saveVaultBackup: vi.fn(), restoreVaultBackup: vi.fn(), exportAccountsFile: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const click = async (name: string) => act(async () => { fireEvent.click(screen.getByRole("button", { name })); });

describe("BackupSettings backups", () => {
  it("says where the backup was saved", async () => {
    vi.mocked(saveVaultBackup).mockResolvedValue("/home/me/b.backup");
    render(<BackupSettings onVaultChanged={() => {}} />);
    await click("Save a backup…");
    expect(screen.getByRole("status").textContent).toContain("/home/me/b.backup");
  });

  it("stays quiet when the file dialog is cancelled", async () => {
    vi.mocked(saveVaultBackup).mockResolvedValue(null);
    render(<BackupSettings onVaultChanged={() => {}} />);
    await click("Save a backup…");
    expect(screen.getByRole("status").textContent).toContain("copy of your encrypted vault");
  });

});

describe("BackupSettings restore", () => {
  it("asks before restoring, starting on Cancel, and refreshes the app once restored", async () => {
    const onVaultChanged = vi.fn();
    vi.mocked(restoreVaultBackup).mockResolvedValue(true);
    render(<BackupSettings onVaultChanged={onVaultChanged} />);
    await click("Restore a backup…");
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Cancel" }));
    expect(restoreVaultBackup).not.toHaveBeenCalled();
    await click("Choose a backup…");
    expect(onVaultChanged).toHaveBeenCalledOnce();
  });

  it("shows why a restore was refused", async () => {
    vi.mocked(restoreVaultBackup).mockRejectedValue("not a DA21 vault");
    render(<BackupSettings onVaultChanged={() => {}} />);
    await click("Restore a backup…");
    await click("Choose a backup…");
    expect(screen.getByRole("status").textContent).toContain("not a DA21 vault");
  });
});

describe("BackupSettings export", () => {
  it("warns, needs the passphrase, and reports how many were exported", async () => {
    vi.mocked(exportAccountsFile).mockResolvedValue(2);
    render(<BackupSettings onVaultChanged={() => {}} />);
    await click("Export accounts…");
    expect(screen.getByText(/unencrypted/)).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Current passphrase"), { target: { value: "my passphrase" } });
    await click("Choose where to save…");
    expect(exportAccountsFile).toHaveBeenCalledWith("my passphrase");
    expect(screen.getByRole("status").textContent).toContain("Exported 2 accounts");
  });

  it("shows a wrong passphrase inside the form and keeps it open", async () => {
    vi.mocked(exportAccountsFile).mockRejectedValue("wrong passphrase");
    render(<BackupSettings onVaultChanged={() => {}} />);
    await click("Export accounts…");
    await click("Choose where to save…");
    expect(screen.getByRole("alert").textContent).toContain("wrong passphrase");
  });
});
