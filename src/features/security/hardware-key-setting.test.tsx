import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { addHardwareKey, disableHardwareKeys, enableHardwareKey, getHardwareKeyStatus, removeHardwareKey, type HardwareKeyStatus } from "@/lib/api";
import { HardwareKeySetting } from "./hardware-key-setting";

vi.mock("@/lib/api", () => ({
  getHardwareKeyStatus: vi.fn(),
  enableHardwareKey: vi.fn(),
  addHardwareKey: vi.fn(),
  removeHardwareKey: vi.fn(),
  disableHardwareKeys: vi.fn(),
}));

const OFF: HardwareKeyStatus = { is_supported: true, is_required: false, keys: [] };
const ON_ONE: HardwareKeyStatus = { is_supported: true, is_required: true, keys: [{ id: "aa", label: "Desk key" }] };
const ON_TWO: HardwareKeyStatus = { is_supported: true, is_required: true, keys: [{ id: "aa", label: "Desk key" }, { id: "bb", label: "Spare" }] };

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function renderSetting(status: HardwareKeyStatus) {
  vi.mocked(getHardwareKeyStatus).mockResolvedValue(status);
  render(<HardwareKeySetting />);
  await act(async () => {});
}

describe("HardwareKeySetting when no key is required", () => {
  it("says so and offers to require one", async () => {
    await renderSetting(OFF);
    expect(screen.getByText("Hardware key: not required")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Require a hardware key" })).toBeTruthy();
  });

  it("reports an unsupported system without offering anything", async () => {
    await renderSetting({ ...OFF, is_supported: false });
    expect(screen.getByText("Hardware keys are not supported on this system yet.")).toBeTruthy();
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("registers a key with the passphrase, an optional PIN and a name", async () => {
    vi.mocked(enableHardwareKey).mockResolvedValue(undefined);
    await renderSetting(OFF);
    fireEvent.click(screen.getByRole("button", { name: "Require a hardware key" }));
    fireEvent.change(screen.getByLabelText("Current passphrase"), { target: { value: "my passphrase" } });
    fireEvent.change(screen.getByLabelText("Key PIN (only if your key has one)"), { target: { value: "1234" } });
    fireEvent.click(screen.getByRole("button", { name: "Register key and require it" }));
    await waitFor(() => expect(enableHardwareKey).toHaveBeenCalledWith("my passphrase", "1234", "Hardware key"));
  });
});

describe("HardwareKeySetting registering a key", () => {
  it("sends no PIN when none is typed and asks the person to touch the key while it waits", async () => {
    vi.mocked(enableHardwareKey).mockReturnValue(new Promise(() => {}));
    await renderSetting(OFF);
    fireEvent.click(screen.getByRole("button", { name: "Require a hardware key" }));
    fireEvent.change(screen.getByLabelText("Current passphrase"), { target: { value: "pw" } });
    fireEvent.click(screen.getByRole("button", { name: "Register key and require it" }));
    expect(await screen.findByText("Touch your key when it blinks…")).toBeTruthy();
    expect(enableHardwareKey).toHaveBeenCalledWith("pw", null, "Hardware key");
  });

  it("shows why it failed and stays on the form", async () => {
    vi.mocked(enableHardwareKey).mockRejectedValue("create a recovery key first, so losing the hardware key cannot lock you out");
    await renderSetting(OFF);
    fireEvent.click(screen.getByRole("button", { name: "Require a hardware key" }));
    fireEvent.click(screen.getByRole("button", { name: "Register key and require it" }));
    expect(await screen.findByText(/create a recovery key first/)).toBeTruthy();
    expect((screen.getByRole("button", { name: "Register key and require it" }) as HTMLButtonElement).disabled).toBe(false);
  });
});

describe("HardwareKeySetting when a key is required", () => {
  it("lists the keys and can add another", async () => {
    vi.mocked(addHardwareKey).mockResolvedValue(undefined);
    await renderSetting(ON_TWO);
    expect(screen.getByText("Hardware key: required")).toBeTruthy();
    expect(screen.getByText("Desk key")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Add another key" }));
    fireEvent.change(screen.getByLabelText("Key name"), { target: { value: "Backup" } });
    fireEvent.click(screen.getByRole("button", { name: "Register key" }));
    await waitFor(() => expect(addHardwareKey).toHaveBeenCalledWith(null, "Backup"));
  });

  it("does not offer to remove the only key", async () => {
    await renderSetting(ON_ONE);
    expect(screen.queryByRole("button", { name: /^Remove/ })).toBeNull();
    expect(screen.getByText(/needs at least one key/)).toBeTruthy();
  });
});

describe("HardwareKeySetting removing and stopping", () => {
  it("asks before removing, naming the key, with Keep key focused", async () => {
    vi.mocked(removeHardwareKey).mockResolvedValue(undefined);
    await renderSetting(ON_TWO);
    fireEvent.click(screen.getByRole("button", { name: "Remove Spare" }));
    expect(screen.getByText("Remove Spare? It will no longer open this vault.")).toBeTruthy();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Keep key" }));
    expect(removeHardwareKey).not.toHaveBeenCalled();
    fireEvent.click(within(screen.getByRole("group", { name: "Remove Spare?" })).getByRole("button", { name: "Remove Spare" }));
    await waitFor(() => expect(removeHardwareKey).toHaveBeenCalledWith("bb"));
  });

  it("backs out of a removal on Escape without removing", async () => {
    await renderSetting(ON_TWO);
    fireEvent.click(screen.getByRole("button", { name: "Remove Spare" }));
    fireEvent.keyDown(screen.getByRole("button", { name: "Keep key" }), { key: "Escape" });
    expect(screen.queryByText(/It will no longer open this vault/)).toBeNull();
    expect(removeHardwareKey).not.toHaveBeenCalled();
  });

  it("stops requiring a key only with the passphrase, and starts on the safe choice", async () => {
    vi.mocked(disableHardwareKeys).mockResolvedValue(undefined);
    await renderSetting(ON_ONE);
    fireEvent.click(screen.getByRole("button", { name: "Stop requiring a key" }));
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Keep requiring a key" }));
    fireEvent.change(screen.getByLabelText("Current passphrase"), { target: { value: "pw" } });
    const stopButtons = screen.getAllByRole("button", { name: "Stop requiring a key" });
    fireEvent.click(stopButtons[stopButtons.length - 1]);
    await waitFor(() => expect(disableHardwareKeys).toHaveBeenCalledWith("pw"));
  });
});
