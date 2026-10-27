import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { resetPassphraseWithRecoveryKey } from "@/lib/api";
import { PASSPHRASE_TOO_SHORT_MESSAGE } from "./passphrase-rules";
import { RecoveryUnlockForm } from "./recovery-unlock-form";
import { UnlockScreen } from "./vault-screens";

vi.mock("@/lib/api", () => ({ unlockNeedsHardwareKey: vi.fn().mockResolvedValue(false), resetPassphraseWithRecoveryKey: vi.fn(), unlockVault: vi.fn(), assessPassphrase: vi.fn(), generatePassphrase: vi.fn(), createVault: vi.fn() }));

const PHRASE = "alpha bravo charlie";
const NEW_ONE = "a brand new passphrase";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function submitWith(passphrase: string, confirmation = passphrase) {
  fireEvent.change(screen.getByLabelText("Recovery key (24 words)"), { target: { value: PHRASE } });
  fireEvent.change(screen.getByLabelText("New passphrase"), { target: { value: passphrase } });
  fireEvent.change(screen.getByLabelText("Confirm new passphrase"), { target: { value: confirmation } });
  fireEvent.click(screen.getByRole("button", { name: "Reset passphrase and unlock" }));
}

describe("RecoveryUnlockForm", () => {
  it("refuses a short new passphrase without asking the backend", async () => {
    render(<RecoveryUnlockForm onUnlocked={() => {}} onBack={() => {}} />);
    submitWith("short");
    expect(await screen.findByText(PASSPHRASE_TOO_SHORT_MESSAGE)).toBeTruthy();
    expect(resetPassphraseWithRecoveryKey).not.toHaveBeenCalled();
  });

  it("sends the words and the new passphrase, then reports the vault unlocked", async () => {
    vi.mocked(resetPassphraseWithRecoveryKey).mockResolvedValue(undefined);
    const onUnlocked = vi.fn();
    render(<RecoveryUnlockForm onUnlocked={onUnlocked} onBack={() => {}} />);
    submitWith(NEW_ONE);
    await waitFor(() => expect(resetPassphraseWithRecoveryKey).toHaveBeenCalledWith(PHRASE, NEW_ONE));
    expect(onUnlocked).toHaveBeenCalled();
  });

  it("shows which word is wrong and stays on the form", async () => {
    vi.mocked(resetPassphraseWithRecoveryKey).mockRejectedValue('"cabage" is not one of the recovery key words; check its spelling');
    const onUnlocked = vi.fn();
    render(<RecoveryUnlockForm onUnlocked={onUnlocked} onBack={() => {}} />);
    submitWith(NEW_ONE);
    expect(await screen.findByText(/"cabage" is not one of the recovery key words/)).toBeTruthy();
    expect(onUnlocked).not.toHaveBeenCalled();
  });
});

describe("UnlockScreen recovery path", () => {
  it("offers the recovery key and can go back to the passphrase", () => {
    render(<UnlockScreen onUnlocked={() => {}} />);
    fireEvent.click(screen.getByRole("button", { name: "Forgot your passphrase? Use your recovery key" }));
    expect(screen.getByText("Use your recovery key")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Back to unlock" }));
    expect(screen.getByText("Unlock DA21")).toBeTruthy();
  });
});
