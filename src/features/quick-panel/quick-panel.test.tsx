import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PrivacySettingsProvider } from "@/features/privacy/privacy-settings-provider";
import { copyCode, hideQuickPanel, listCodes, lockVault, openAppPage, type CodeView, type VendorId } from "@/lib/api";
import { DEFAULT_SETTINGS } from "@/test-support/settings-fixture";
import { QuickPanel } from "./quick-panel";

vi.mock("@/lib/api", () => ({
  listCodes: vi.fn(),
  copyCode: vi.fn(),
  hideQuickPanel: vi.fn().mockResolvedValue(undefined),
  openAppPage: vi.fn().mockResolvedValue(undefined),
  lockVault: vi.fn().mockResolvedValue(undefined),
  getSettings: vi.fn(),
  getVendors: vi.fn().mockResolvedValue([
    { id: "aegis", name: "Aegis" },
    { id: "authy", name: "Authy" },
    { id: "other", name: "Other" },
  ]),
}));

const account = (issuer: string, vendor: VendorId): CodeView => ({
  id: issuer, label: `${issuer} (me)`, issuer, name: "me", vendor, code: "482913", seconds_left: 12, period_seconds: 30,
});
const ACCOUNTS = [account("GitHub", "aegis"), account("Amazon", "authy"), account("Slack", "authy")];

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function renderPanel(codes: CodeView[] | "locked" = ACCOUNTS, hideCodes = true) {
  if (codes === "locked") vi.mocked(listCodes).mockRejectedValue("the vault is locked");
  else vi.mocked(listCodes).mockResolvedValue(codes);
  vi.mocked((await import("@/lib/api")).getSettings).mockResolvedValue({ ...DEFAULT_SETTINGS, hide_codes: hideCodes });
  render(<PrivacySettingsProvider><QuickPanel /></PrivacySettingsProvider>);
  await act(async () => {});
}

describe("QuickPanel list", () => {
  it("shows every account under a heading for its app, with full names", async () => {
    await renderPanel();
    expect(screen.getByRole("heading", { name: "Aegis (1)" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Authy (2)" })).toBeTruthy();
    expect(screen.getAllByRole("button", { name: /^Copy code for/ })).toHaveLength(3);
  });

  it("shows more accounts than the old menu's limit of eight", async () => {
    const many = Array.from(Array(30).keys(), (index) => account(`Service ${index}`, "other"));
    await renderPanel(many);
    expect(screen.getAllByRole("button", { name: /^Copy code for/ })).toHaveLength(30);
  });

  it("keeps codes hidden until revealed, like the window", async () => {
    await renderPanel();
    expect(document.body.textContent).not.toContain("482913");
    fireEvent.click(screen.getByRole("button", { name: "Reveal code for GitHub (me)" }));
    expect(document.body.textContent).toContain("482913");
  });
});

describe("QuickPanel search", () => {
  it("has the cursor in the search box and narrows the list as you type", async () => {
    await renderPanel();
    const search = screen.getByRole("searchbox", { name: "Search accounts" });
    expect(document.activeElement).toBe(search);
    fireEvent.change(search, { target: { value: "sla" } });
    expect(screen.getAllByRole("button", { name: /^Copy code for/ })).toHaveLength(1);
    expect(screen.queryByRole("heading", { name: /Aegis/ })).toBeNull();
  });

  it("says when nothing matches", async () => {
    await renderPanel();
    fireEvent.change(screen.getByRole("searchbox"), { target: { value: "zzz" } });
    expect(screen.getByText('No accounts match "zzz".')).toBeTruthy();
  });

  it("copies the first match on Enter and closes the panel", async () => {
    vi.mocked(copyCode).mockResolvedValue(undefined);
    await renderPanel();
    fireEvent.change(screen.getByRole("searchbox"), { target: { value: "am" } });
    fireEvent.keyDown(screen.getByRole("searchbox"), { key: "Enter" });
    await waitFor(() => expect(copyCode).toHaveBeenCalledWith("Amazon"));
    await waitFor(() => expect(hideQuickPanel).toHaveBeenCalled());
  });
});

describe("QuickPanel keys", () => {
  it("Escape clears the search first and closes the panel the second time", async () => {
    await renderPanel();
    const search = screen.getByRole("searchbox") as HTMLInputElement;
    fireEvent.change(search, { target: { value: "git" } });
    fireEvent.keyDown(search, { key: "Escape" });
    expect(search.value).toBe("");
    expect(hideQuickPanel).not.toHaveBeenCalled();
    fireEvent.keyDown(search, { key: "Escape" });
    expect(hideQuickPanel).toHaveBeenCalled();
  });

  it("the arrow keys move between the search box and the rows", async () => {
    await renderPanel();
    const search = screen.getByRole("searchbox");
    fireEvent.keyDown(search, { key: "ArrowDown" });
    const rows = screen.getAllByRole("button", { name: /^Copy code for/ });
    expect(document.activeElement).toBe(rows[0]);
    fireEvent.keyDown(rows[0], { key: "ArrowDown" });
    expect(document.activeElement).toBe(rows[1]);
    fireEvent.keyDown(rows[1], { key: "ArrowUp" });
    fireEvent.keyDown(rows[0], { key: "ArrowUp" });
    expect(document.activeElement).toBe(search);
  });
});

describe("QuickPanel clicks and footer", () => {
  it("copies a clicked row and closes the panel", async () => {
    vi.mocked(copyCode).mockResolvedValue(undefined);
    await renderPanel();
    fireEvent.click(screen.getByRole("button", { name: "Copy code for Slack (me)" }));
    await waitFor(() => expect(copyCode).toHaveBeenCalledWith("Slack"));
    await waitFor(() => expect(hideQuickPanel).toHaveBeenCalled());
  });

  it("opens the app, opens the settings and locks", async () => {
    await renderPanel();
    fireEvent.click(screen.getByRole("button", { name: "Open DeskAuth21" }));
    fireEvent.click(screen.getByRole("button", { name: "Open settings" }));
    expect(openAppPage).toHaveBeenNthCalledWith(1, "codes");
    expect(openAppPage).toHaveBeenNthCalledWith(2, "settings");
    fireEvent.click(screen.getByRole("button", { name: "Lock now" }));
    await waitFor(() => expect(lockVault).toHaveBeenCalled());
  });
});

describe("QuickPanel when locked", () => {
  it("shows no accounts and offers to open the app to unlock", async () => {
    await renderPanel("locked");
    expect(screen.getByText("DeskAuth21 is locked.")).toBeTruthy();
    expect(screen.queryByRole("searchbox")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Open DeskAuth21 to unlock" }));
    expect(openAppPage).toHaveBeenCalledWith("codes");
  });
});
