import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PrivacySettingsProvider } from "@/features/privacy/privacy-settings-provider";
import { copyCode, getSettings, type CodeView } from "@/lib/api";
import { DEFAULT_SETTINGS } from "@/test-support/settings-fixture";
import { CodeRow } from "./code-row";

vi.mock("@/lib/api", () => ({ copyCode: vi.fn(), getSettings: vi.fn() }));

const VIEW: CodeView = { id: "acct-1", label: "Acme (a@b)", issuer: "Acme", name: "a@b", vendor: "other", code: "123456", seconds_left: 12, period_seconds: 30 };
const MASKED = "••••••";

async function renderRow(hideCodes: boolean) {
  vi.mocked(getSettings).mockResolvedValue({ ...DEFAULT_SETTINGS, hide_codes: hideCodes });
  render(<PrivacySettingsProvider><ul><CodeRow view={VIEW} /></ul></PrivacySettingsProvider>);
  await act(async () => {});
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("CodeRow with codes shown", () => {
  it("shows the label, code and countdown, and has no eye button", async () => {
    await renderRow(false);
    expect(screen.getByText("Acme")).toBeTruthy();
    expect(screen.getByText("a@b")).toBeTruthy();
    expect(screen.getByText("123456")).toBeTruthy();
    expect(screen.getByRole("img", { name: "12 seconds left" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Reveal code/ })).toBeNull();
  });

  it("asks the backend to copy this account and confirms", async () => {
    vi.mocked(copyCode).mockResolvedValue(undefined);
    await renderRow(false);
    fireEvent.click(screen.getByRole("button", { name: "Copy code for Acme (a@b)" }));
    await waitFor(() => expect(copyCode).toHaveBeenCalledWith("acct-1"));
    expect((await screen.findAllByText("Copied")).length).toBeGreaterThan(0);
  });

  it("reports a failed copy", async () => {
    vi.mocked(copyCode).mockRejectedValue("clipboard unavailable");
    await renderRow(false);
    fireEvent.click(screen.getByRole("button", { name: /Copy code for/ }));
    expect((await screen.findAllByText("Copy failed")).length).toBeGreaterThan(0);
  });
});

describe("CodeRow with codes hidden", () => {
  it("shows dots, not digits, anywhere in the page", async () => {
    await renderRow(true);
    expect(screen.getByText(MASKED)).toBeTruthy();
    expect(document.body.textContent).not.toContain("123456");
  });

  it("reveals on the eye button and hides again on the same button", async () => {
    await renderRow(true);
    fireEvent.click(screen.getByRole("button", { name: "Reveal code for Acme (a@b)" }));
    expect(screen.getByText("123456")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Hide code for Acme (a@b)" }));
    expect(screen.getByText(MASKED)).toBeTruthy();
  });

  it("copies the code without revealing it", async () => {
    vi.mocked(copyCode).mockResolvedValue(undefined);
    await renderRow(true);
    fireEvent.click(screen.getByRole("button", { name: "Copy code for Acme (a@b)" }));
    await waitFor(() => expect(copyCode).toHaveBeenCalledWith("acct-1"));
    expect(document.body.textContent).not.toContain("123456");
  });

  it("hides again when the window loses focus", async () => {
    await renderRow(true);
    fireEvent.click(screen.getByRole("button", { name: "Reveal code for Acme (a@b)" }));
    act(() => window.dispatchEvent(new Event("blur")));
    expect(screen.getByText(MASKED)).toBeTruthy();
  });
});
