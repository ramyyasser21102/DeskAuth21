import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { generateRecoveryKey, saveRecoveryKey } from "@/lib/api";
import { RecoveryKeyWizard } from "./recovery-key-wizard";

vi.mock("@/lib/api", () => ({ generateRecoveryKey: vi.fn(), saveRecoveryKey: vi.fn() }));

const WORDS = Array.from(Array(24).keys(), (index) => `word${index + 1}`);
const PHRASE = WORDS.join(" ");

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function openConfirmStep(onClose = vi.fn()) {
  vi.mocked(generateRecoveryKey).mockResolvedValue(PHRASE);
  render(<RecoveryKeyWizard onClose={onClose} />);
  await screen.findByRole("list", { name: "Recovery key words" });
  fireEvent.click(screen.getByRole("button", { name: "I have written it down" }));
  return onClose;
}

const askedWords = () => screen.getAllByLabelText(/^Word \d+$/).map((field) => field as HTMLInputElement);
const wordFor = (field: HTMLInputElement) => WORDS[Number(field.id.replace("recovery-word-", ""))];

describe("RecoveryKeyWizard showing the key", () => {
  it("shows the 24 words numbered and warns they are shown once, saving nothing yet", async () => {
    vi.mocked(generateRecoveryKey).mockResolvedValue(PHRASE);
    render(<RecoveryKeyWizard onClose={() => {}} />);
    const list = await screen.findByRole("list", { name: "Recovery key words" });
    expect(list.querySelectorAll("li")).toHaveLength(24);
    expect(screen.getByText(/shown only once/)).toBeTruthy();
    expect(saveRecoveryKey).not.toHaveBeenCalled();
  });

  it("closes without saving when cancelled", async () => {
    vi.mocked(generateRecoveryKey).mockResolvedValue(PHRASE);
    const onClose = vi.fn();
    render(<RecoveryKeyWizard onClose={onClose} />);
    await screen.findByRole("list");
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onClose).toHaveBeenCalled();
    expect(saveRecoveryKey).not.toHaveBeenCalled();
  });
});

describe("RecoveryKeyWizard confirming the key", () => {
  it("asks for three of the words and refuses wrong ones without saving", async () => {
    await openConfirmStep();
    const fields = askedWords();
    expect(fields).toHaveLength(3);
    fields.forEach((field) => fireEvent.change(field, { target: { value: "nope" } }));
    fireEvent.change(screen.getByLabelText("Current passphrase"), { target: { value: "pw" } });
    fireEvent.click(screen.getByRole("button", { name: "Save recovery key" }));
    expect(await screen.findByText(/do not match the ones shown/)).toBeTruthy();
    expect(saveRecoveryKey).not.toHaveBeenCalled();
  });

  it("saves the key with the current passphrase once the words match", async () => {
    vi.mocked(saveRecoveryKey).mockResolvedValue(undefined);
    await openConfirmStep();
    askedWords().forEach((field) => fireEvent.change(field, { target: { value: wordFor(field).toUpperCase() } }));
    fireEvent.change(screen.getByLabelText("Current passphrase"), { target: { value: "my passphrase" } });
    fireEvent.click(screen.getByRole("button", { name: "Save recovery key" }));
    await waitFor(() => expect(saveRecoveryKey).toHaveBeenCalledWith(PHRASE, "my passphrase"));
    expect(await screen.findByText(/Recovery key saved/)).toBeTruthy();
  });

  it("shows a wrong passphrase from the backend and stays on the step", async () => {
    vi.mocked(saveRecoveryKey).mockRejectedValue("wrong passphrase or damaged vault file");
    await openConfirmStep();
    askedWords().forEach((field) => fireEvent.change(field, { target: { value: wordFor(field) } }));
    await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Save recovery key" })); });
    expect(await screen.findByText("wrong passphrase or damaged vault file")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Save recovery key" })).toBeTruthy();
  });
});
