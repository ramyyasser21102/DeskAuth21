import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { assessPassphrase, changePassphrase, generatePassphrase } from "@/lib/api";
import { PASSPHRASE_TOO_SHORT_MESSAGE } from "@/features/vault/passphrase-rules";
import { ChangePassphraseForm } from "./change-passphrase-form";

vi.mock("@/lib/api", () => ({ changePassphrase: vi.fn(), assessPassphrase: vi.fn(), generatePassphrase: vi.fn() }));

const NEW_ONE = "a brand new passphrase";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function fill(current: string, passphrase: string, confirmation: string) {
  fireEvent.change(screen.getByLabelText("Current passphrase"), { target: { value: current } });
  fireEvent.change(screen.getByLabelText("New passphrase"), { target: { value: passphrase } });
  fireEvent.change(screen.getByLabelText("Confirm new passphrase"), { target: { value: confirmation } });
  fireEvent.click(screen.getByRole("button", { name: "Change passphrase" }));
}

describe("ChangePassphraseForm", () => {
  it("refuses a short new passphrase without asking the backend", async () => {
    render(<ChangePassphraseForm onClose={() => {}} />);
    fill("old one", "short", "short");
    expect(await screen.findByText(PASSPHRASE_TOO_SHORT_MESSAGE)).toBeTruthy();
    expect(changePassphrase).not.toHaveBeenCalled();
  });

  it("sends the current and new passphrase and says it worked", async () => {
    vi.mocked(changePassphrase).mockResolvedValue(undefined);
    render(<ChangePassphraseForm onClose={() => {}} />);
    fill("old passphrase", NEW_ONE, NEW_ONE);
    await waitFor(() => expect(changePassphrase).toHaveBeenCalledWith("old passphrase", NEW_ONE));
    expect(await screen.findByText(/Passphrase changed/)).toBeTruthy();
    expect((screen.getByLabelText("New passphrase") as HTMLInputElement).value).toBe("");
  });

  it("shows a wrong current passphrase from the backend", async () => {
    vi.mocked(changePassphrase).mockRejectedValue("wrong passphrase or damaged vault file");
    render(<ChangePassphraseForm onClose={() => {}} />);
    await act(async () => { fill("not it", NEW_ONE, NEW_ONE); });
    expect(await screen.findByText("wrong passphrase or damaged vault file")).toBeTruthy();
  });

  it("fills both new fields from a generated passphrase", async () => {
    vi.mocked(generatePassphrase).mockResolvedValue("alpha-bravo-charlie-delta-echo-foxtrot");
    vi.mocked(assessPassphrase).mockResolvedValue({ score: 4, feedback: null });
    render(<ChangePassphraseForm onClose={() => {}} />);
    fireEvent.click(screen.getByRole("button", { name: "Generate a passphrase" }));
    await screen.findByText("alpha-bravo-charlie-delta-echo-foxtrot");
    expect((screen.getByLabelText("Confirm new passphrase") as HTMLInputElement).value).toBe("alpha-bravo-charlie-delta-echo-foxtrot");
  });
});
