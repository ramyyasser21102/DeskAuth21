import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { chooseVaultFolder, getVaultFolder } from "@/lib/api";
import { UseExistingVaultButton } from "./use-existing-vault-button";
import { VaultFolderSetting } from "./vault-folder-setting";

vi.mock("@/lib/api", () => ({ getVaultFolder: vi.fn(), chooseVaultFolder: vi.fn() }));

beforeEach(() => {
  vi.mocked(getVaultFolder).mockResolvedValue("/Users/me/Dropbox/DeskAuth21");
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const clickChoose = async () => fireEvent.click(await screen.findByRole("button", { name: /Choose folder/ }));

describe("VaultFolderSetting display", () => {
  it("shows where the vault is and how syncing works", async () => {
    render(<VaultFolderSetting onVaultChanged={() => {}} />);
    expect(await screen.findByText("/Users/me/Dropbox/DeskAuth21")).toBeTruthy();
    expect(screen.getByText(/Only the encrypted file is shared/)).toBeTruthy();
  });

  it("says nothing when the picker is cancelled", async () => {
    vi.mocked(chooseVaultFolder).mockResolvedValue(null);
    render(<VaultFolderSetting onVaultChanged={() => {}} />);
    await clickChoose();
    await waitFor(() => expect(chooseVaultFolder).toHaveBeenCalled());
    expect(screen.queryByText(/Vault moved|Now using/)).toBeNull();
  });

  it("shows an error from the backend", async () => {
    vi.mocked(chooseVaultFolder).mockRejectedValue("a folder whose name is not valid text is not an allowed setting");
    render(<VaultFolderSetting onVaultChanged={() => {}} />);
    await clickChoose();
    expect(await screen.findByText(/not an allowed setting/)).toBeTruthy();
  });
});

describe("VaultFolderSetting outcomes", () => {
  it("explains a move and does not ask to unlock again", async () => {
    vi.mocked(chooseVaultFolder).mockResolvedValue("moved");
    const onVaultChanged = vi.fn();
    render(<VaultFolderSetting onVaultChanged={onVaultChanged} />);
    await clickChoose();
    expect(await screen.findByText(/Vault moved/)).toBeTruthy();
    expect(onVaultChanged).not.toHaveBeenCalled();
  });

  it("returns to the unlock screen after adopting an existing vault", async () => {
    vi.mocked(chooseVaultFolder).mockResolvedValue("used_existing");
    const onVaultChanged = vi.fn();
    render(<VaultFolderSetting onVaultChanged={onVaultChanged} />);
    await clickChoose();
    await waitFor(() => expect(onVaultChanged).toHaveBeenCalledTimes(1));
  });
});

describe("UseExistingVaultButton", () => {
  it("continues only when a folder was actually chosen", async () => {
    const onChosen = vi.fn();
    vi.mocked(chooseVaultFolder).mockResolvedValueOnce(null).mockResolvedValueOnce("used_existing");
    render(<UseExistingVaultButton onChosen={onChosen} />);
    const button = screen.getByRole("button", { name: /Already have a vault/ });
    fireEvent.click(button);
    await waitFor(() => expect(chooseVaultFolder).toHaveBeenCalledTimes(1));
    expect(onChosen).not.toHaveBeenCalled();
    fireEvent.click(button);
    await waitFor(() => expect(onChosen).toHaveBeenCalledTimes(1));
  });

  it("shows the error when choosing fails", async () => {
    vi.mocked(chooseVaultFolder).mockRejectedValue("no permission");
    render(<UseExistingVaultButton onChosen={() => {}} />);
    fireEvent.click(screen.getByRole("button"));
    expect(await screen.findByText("no permission")).toBeTruthy();
  });
});
