import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { listCodes, type CodeView } from "@/lib/api";
import { CodesScreen } from "./codes-screen";

vi.mock("@/lib/api", () => ({
  listCodes: vi.fn(),
  lockVault: vi.fn(),
  copyCode: vi.fn(),
  importText: vi.fn(),
  getSettings: vi.fn().mockResolvedValue({ auto_lock_minutes: 5 }),
  setAutoLockMinutes: vi.fn(),
  recordActivity: vi.fn().mockResolvedValue(undefined),
  getVaultFolder: vi.fn().mockResolvedValue("/data/DeskAuth21"),
  chooseVaultFolder: vi.fn(),
}));

const ACME: CodeView = { id: "acct-1", label: "Acme (a@b)", issuer: "Acme", name: "a@b", vendor: "other", code: "123456", seconds_left: 10 };
const IMPORT_BUTTON_LABEL = "Choose files…";
const POLL_WAIT_MS = 3000;

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("CodesScreen import panel", () => {
  it("opens for an empty vault and stays open after the first account appears", async () => {
    vi.mocked(listCodes).mockResolvedValueOnce([]).mockResolvedValue([ACME]);
    render(<CodesScreen onLocked={() => {}} />);
    expect(await screen.findByText(IMPORT_BUTTON_LABEL)).toBeTruthy();
    await waitFor(() => expect(screen.getByText("Acme (a@b)")).toBeTruthy(), { timeout: POLL_WAIT_MS });
    expect(screen.getByText(IMPORT_BUTTON_LABEL)).toBeTruthy();
  });

  it("stays closed for a vault that already has accounts", async () => {
    vi.mocked(listCodes).mockResolvedValue([ACME]);
    render(<CodesScreen onLocked={() => {}} />);
    expect(await screen.findByText("Acme (a@b)")).toBeTruthy();
    expect(screen.queryByText(IMPORT_BUTTON_LABEL)).toBeNull();
    expect(screen.getByRole("button", { name: "Add accounts" })).toBeTruthy();
  });
});
