import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { assessPassphrase, generatePassphrase } from "@/lib/api";
import { PassphraseForm } from "./passphrase-form";
import { PASSPHRASE_MISMATCH_MESSAGE, PASSPHRASE_TOO_SHORT_MESSAGE } from "./passphrase-rules";

vi.mock("@/lib/api", () => ({ assessPassphrase: vi.fn(), generatePassphrase: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const LONG = "correct-horse-battery";

function fillAndSubmit(fields: Record<string, string>) {
  for (const [label, value] of Object.entries(fields)) {
    fireEvent.change(screen.getByLabelText(label), { target: { value } });
  }
  fireEvent.submit(screen.getByRole("button", { name: "Go" }).closest("form")!);
}

const renderCreate = (onSubmit = vi.fn().mockResolvedValue(undefined)) => {
  render(<PassphraseForm title="Create" submitLabel="Go" requiresConfirmation onSubmit={onSubmit} />);
  return onSubmit;
};

describe("PassphraseForm rules when creating", () => {
  it("blocks a passphrase that is too short", async () => {
    const onSubmit = renderCreate();
    fillAndSubmit({ Passphrase: "short", "Confirm passphrase": "short" });
    expect(await screen.findByText(PASSPHRASE_TOO_SHORT_MESSAGE)).toBeTruthy();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("blocks submission when the confirmation does not match", async () => {
    const onSubmit = renderCreate();
    fillAndSubmit({ Passphrase: LONG, "Confirm passphrase": `${LONG}-x` });
    expect(await screen.findByText(PASSPHRASE_MISMATCH_MESSAGE)).toBeTruthy();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("submits the passphrase when everything matches", async () => {
    const onSubmit = renderCreate();
    fillAndSubmit({ Passphrase: LONG, "Confirm passphrase": LONG });
    await waitFor(() => expect(onSubmit).toHaveBeenCalledWith(LONG));
  });

});

describe("PassphraseForm helpers when creating", () => {
  it("fills both fields with a generated passphrase and shows it", async () => {
    vi.mocked(generatePassphrase).mockResolvedValue("alpha-bravo-charlie-delta-echo-foxtrot");
    const onSubmit = renderCreate();
    fireEvent.click(screen.getByRole("button", { name: "Generate a passphrase" }));
    expect(await screen.findByText("alpha-bravo-charlie-delta-echo-foxtrot")).toBeTruthy();
    fireEvent.submit(screen.getByRole("button", { name: "Go" }).closest("form")!);
    await waitFor(() => expect(onSubmit).toHaveBeenCalledWith("alpha-bravo-charlie-delta-echo-foxtrot"));
  });

  it("rates what is typed once the person pauses", async () => {
    vi.useFakeTimers();
    vi.mocked(assessPassphrase).mockResolvedValue({ score: 3, feedback: null });
    renderCreate();
    fireEvent.change(screen.getByLabelText("Passphrase"), { target: { value: LONG } });
    await act(async () => { await vi.advanceTimersByTimeAsync(300); });
    vi.useRealTimers();
    expect(screen.getByRole("meter", { name: "Passphrase strength" }).getAttribute("aria-valuetext")).toBe("Good");
  });
});

describe("PassphraseForm when unlocking", () => {
  it("accepts any length, shows no meter or generator, and shows the backend error", async () => {
    const onSubmit = vi.fn().mockRejectedValue("wrong passphrase or damaged vault file");
    render(<PassphraseForm title="Unlock" submitLabel="Go" requiresConfirmation={false} onSubmit={onSubmit} />);
    expect(screen.queryByRole("button", { name: "Generate a passphrase" })).toBeNull();
    fillAndSubmit({ Passphrase: "nope" });
    expect(await screen.findByText("wrong passphrase or damaged vault file")).toBeTruthy();
    expect(onSubmit).toHaveBeenCalledWith("nope");
    await waitFor(() => expect((screen.getByRole("button", { name: "Go" }) as HTMLButtonElement).disabled).toBe(false));
  });
});
