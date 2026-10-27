import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PassphraseForm } from "./passphrase-form";
import { PASSPHRASE_MISMATCH_MESSAGE } from "./passphrase-rules";

afterEach(cleanup);

function fillAndSubmit(fields: Record<string, string>) {
  for (const [label, value] of Object.entries(fields)) {
    fireEvent.change(screen.getByLabelText(label), { target: { value } });
  }
  fireEvent.submit(screen.getByRole("button").closest("form")!);
}

describe("PassphraseForm", () => {
  it("blocks submission when the confirmation does not match", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    render(<PassphraseForm title="Create" submitLabel="Go" requiresConfirmation onSubmit={onSubmit} />);
    fillAndSubmit({ Passphrase: "first-one", "Confirm passphrase": "second-one" });
    expect(await screen.findByText(PASSPHRASE_MISMATCH_MESSAGE)).toBeTruthy();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("submits the passphrase when everything matches", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    render(<PassphraseForm title="Create" submitLabel="Go" requiresConfirmation onSubmit={onSubmit} />);
    fillAndSubmit({ Passphrase: "same-one", "Confirm passphrase": "same-one" });
    await waitFor(() => expect(onSubmit).toHaveBeenCalledWith("same-one"));
  });

  it("shows the backend error and keeps the form usable", async () => {
    const onSubmit = vi.fn().mockRejectedValue("wrong passphrase or corrupted vault");
    render(<PassphraseForm title="Unlock" submitLabel="Go" requiresConfirmation={false} onSubmit={onSubmit} />);
    fillAndSubmit({ Passphrase: "nope" });
    expect(await screen.findByText("wrong passphrase or corrupted vault")).toBeTruthy();
    await waitFor(() => expect((screen.getByRole("button") as HTMLButtonElement).disabled).toBe(false));
  });
});
