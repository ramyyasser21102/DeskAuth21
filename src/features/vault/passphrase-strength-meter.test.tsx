import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { assessPassphrase } from "@/lib/api";
import { PassphraseStrengthMeter } from "./passphrase-strength-meter";

vi.mock("@/lib/api", () => ({ assessPassphrase: vi.fn() }));

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.clearAllMocks();
});

const settle = () => act(async () => { await vi.advanceTimersByTimeAsync(300); });

describe("PassphraseStrengthMeter", () => {
  it("shows nothing until there is something to rate", async () => {
    render(<PassphraseStrengthMeter passphrase="" />);
    await settle();
    expect(screen.queryByRole("meter")).toBeNull();
    expect(assessPassphrase).not.toHaveBeenCalled();
  });

  it("shows the label and the advice for a weak passphrase", async () => {
    vi.mocked(assessPassphrase).mockResolvedValue({ score: 1, feedback: "This is a very common password." });
    render(<PassphraseStrengthMeter passphrase="password1234" />);
    await settle();
    const meter = screen.getByRole("meter", { name: "Passphrase strength" });
    expect(meter.getAttribute("aria-valuenow")).toBe("1");
    expect(screen.getByText("Weak. This is a very common password.")).toBeTruthy();
  });

  it("waits for a pause in typing and rates only the latest text", async () => {
    vi.mocked(assessPassphrase).mockResolvedValue({ score: 4, feedback: null });
    const { rerender } = render(<PassphraseStrengthMeter passphrase="a" />);
    rerender(<PassphraseStrengthMeter passphrase="ab" />);
    rerender(<PassphraseStrengthMeter passphrase="abc" />);
    await settle();
    expect(assessPassphrase).toHaveBeenCalledTimes(1);
    expect(assessPassphrase).toHaveBeenCalledWith("abc");
    expect(screen.getByText("Strong")).toBeTruthy();
  });
});
