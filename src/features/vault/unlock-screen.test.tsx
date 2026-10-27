import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { unlockNeedsHardwareKey, unlockVault } from "@/lib/api";
import { UnlockScreen } from "./vault-screens";

vi.mock("@/lib/api", () => ({ unlockVault: vi.fn(), unlockNeedsHardwareKey: vi.fn(), createVault: vi.fn(), resetPassphraseWithRecoveryKey: vi.fn(), assessPassphrase: vi.fn(), generatePassphrase: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function renderUnlock(needsKey: boolean, onUnlocked = vi.fn()) {
  vi.mocked(unlockNeedsHardwareKey).mockResolvedValue(needsKey);
  render(<UnlockScreen onUnlocked={onUnlocked} />);
  await act(async () => {});
  return onUnlocked;
}

const submit = (passphrase: string) => {
  fireEvent.change(screen.getByLabelText("Passphrase"), { target: { value: passphrase } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock" }));
};

describe("UnlockScreen without a hardware key", () => {
  it("shows no key step and unlocks with the passphrase alone", async () => {
    vi.mocked(unlockVault).mockResolvedValue(undefined);
    const onUnlocked = await renderUnlock(false);
    expect(screen.queryByLabelText(/Key PIN/)).toBeNull();
    submit("my passphrase");
    await waitFor(() => expect(unlockVault).toHaveBeenCalledWith("my passphrase", undefined));
    expect(onUnlocked).toHaveBeenCalled();
  });
});

describe("UnlockScreen with a hardware key required", () => {
  it("tells the person to plug the key in and offers a PIN field", async () => {
    await renderUnlock(true);
    expect(screen.getByText(/also needs your hardware key/)).toBeTruthy();
    expect(screen.getByLabelText("Key PIN (only if your key has one)")).toBeTruthy();
  });

  it("sends the PIN with the passphrase and asks for a touch while it waits", async () => {
    vi.mocked(unlockVault).mockReturnValue(new Promise(() => {}));
    await renderUnlock(true);
    fireEvent.change(screen.getByLabelText("Key PIN (only if your key has one)"), { target: { value: "4321" } });
    submit("my passphrase");
    expect(await screen.findByText("Touch your key when it blinks…")).toBeTruthy();
    expect(unlockVault).toHaveBeenCalledWith("my passphrase", "4321");
  });

  it("shows the reason when the key is missing and stays usable", async () => {
    vi.mocked(unlockVault).mockRejectedValue("no hardware key was found; plug it in and try again");
    await renderUnlock(true);
    submit("my passphrase");
    expect(await screen.findByText(/no hardware key was found/)).toBeTruthy();
    await waitFor(() => expect((screen.getByRole("button", { name: "Unlock" }) as HTMLButtonElement).disabled).toBe(false));
  });
});
