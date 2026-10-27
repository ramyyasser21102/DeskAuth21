import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { listCodes, type CodeView, type VendorId } from "@/lib/api";
import { UnlockedApp } from "./unlocked-app";

vi.mock("@/lib/api", () => ({
  listCodes: vi.fn(),
  lockVault: vi.fn(),
  copyCode: vi.fn(),
  importText: vi.fn(),
  getVendors: vi.fn().mockResolvedValue([
    { id: "aegis", name: "Aegis" },
    { id: "authy", name: "Authy" },
    { id: "other", name: "Other" },
  ]),
  getSettings: vi.fn().mockResolvedValue({ auto_lock_minutes: 5 }),
  setAutoLockMinutes: vi.fn(),
  recordActivity: vi.fn().mockResolvedValue(undefined),
  getVaultFolder: vi.fn().mockResolvedValue("/data/DeskAuth21"),
  chooseVaultFolder: vi.fn(),
}));

const account = (id: string, vendor: VendorId): CodeView => ({
  id, label: id, issuer: id, name: id, vendor, code: "123456", seconds_left: 10, period_seconds: 30,
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const openTab = (name: string) => fireEvent.click(screen.getByRole("button", { name }));

describe("UnlockedApp pages", () => {
  it("starts on the codes page and moves between pages with the tab bar", async () => {
    vi.mocked(listCodes).mockResolvedValue([account("Acme", "aegis")]);
    render(<UnlockedApp onLocked={() => {}} />);
    expect(await screen.findByText("Your codes")).toBeTruthy();

    openTab("Import");
    expect(screen.getByText("Import accounts")).toBeTruthy();
    expect(screen.getByLabelText("Files to import")).toBeTruthy();
    expect(screen.queryByText("Your codes")).toBeNull();

    openTab("Settings");
    expect(screen.getByText("Settings", { selector: "[data-slot=card-title]" })).toBeTruthy();
    expect(await screen.findByRole("button", { name: "Lock now" })).toBeTruthy();
  });

  it("offers to import from the empty codes page", async () => {
    vi.mocked(listCodes).mockResolvedValue([]);
    render(<UnlockedApp onLocked={() => {}} />);
    fireEvent.click(await screen.findByRole("button", { name: "Import accounts" }));
    expect(screen.getByLabelText("Files to import")).toBeTruthy();
  });
});

describe("UnlockedApp vendor tabs", () => {
  const mixed = [account("Aegis one", "aegis"), account("Authy one", "authy"), account("Authy two", "authy")];

  it("filters the list by app and goes back to all", async () => {
    vi.mocked(listCodes).mockResolvedValue(mixed);
    render(<UnlockedApp onLocked={() => {}} />);
    fireEvent.click(await screen.findByRole("button", { name: "Authy (2)" }));
    expect(screen.queryByText("Aegis one")).toBeNull();
    expect(screen.getByText("Authy two")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "All (3)" }));
    await waitFor(() => expect(screen.getByText("Aegis one")).toBeTruthy());
  });

  it("hides the chips when every account is from one app", async () => {
    vi.mocked(listCodes).mockResolvedValue([account("Only", "aegis")]);
    render(<UnlockedApp onLocked={() => {}} />);
    expect(await screen.findByText("Only")).toBeTruthy();
    expect(screen.queryByRole("group", { name: "Filter by app" })).toBeNull();
  });
});
