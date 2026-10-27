import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { deleteAccount, renameAccount, type CodeView } from "@/lib/api";
import { EMPTY_NAME_MESSAGE } from "./account-rules";
import { CodeRow } from "./code-row";

vi.mock("@/lib/api", () => ({ copyCode: vi.fn(), renameAccount: vi.fn(), deleteAccount: vi.fn() }));

const VIEW: CodeView = { id: "acct-1", label: "Acme (a@b)", issuer: "Acme", name: "a@b", code: "123456", seconds_left: 12 };

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function openEditor() {
  render(<ul><CodeRow view={VIEW} /></ul>);
  fireEvent.click(screen.getByRole("button", { name: "Edit Acme (a@b)" }));
}

describe("account editor: saving", () => {
  it("starts with the current labels and saves changes by id", async () => {
    vi.mocked(renameAccount).mockResolvedValue(undefined);
    openEditor();
    expect((screen.getByLabelText("Service") as HTMLInputElement).value).toBe("Acme");
    fireEvent.change(screen.getByLabelText("Account name"), { target: { value: "me@work.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(renameAccount).toHaveBeenCalledWith("acct-1", { issuer: "Acme", name: "me@work.com" }));
    await waitFor(() => expect(screen.queryByLabelText("Service")).toBeNull());
  });

});

describe("account editor: leaving without saving", () => {
  it("will not save an empty name", async () => {
    openEditor();
    fireEvent.change(screen.getByLabelText("Account name"), { target: { value: "  " } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText(EMPTY_NAME_MESSAGE)).toBeTruthy();
    expect(renameAccount).not.toHaveBeenCalled();
  });

  it("shows a backend error and stays open", async () => {
    vi.mocked(renameAccount).mockRejectedValue("vault is locked");
    openEditor();
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("vault is locked")).toBeTruthy();
    expect(screen.getByLabelText("Service")).toBeTruthy();
  });

  it("cancels without saving", () => {
    openEditor();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(renameAccount).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Edit Acme (a@b)" })).toBeTruthy();
  });
});

describe("account editor: deleting", () => {
  it("asks for confirmation and warns about losing access before deleting", async () => {
    vi.mocked(deleteAccount).mockResolvedValue(undefined);
    openEditor();
    fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    expect(screen.getByText("Delete Acme (a@b)?")).toBeTruthy();
    expect(screen.getByText(/could lose access/)).toBeTruthy();
    expect(deleteAccount).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Delete account" }));
    await waitFor(() => expect(deleteAccount).toHaveBeenCalledWith("acct-1"));
  });

  it("can back out of a delete", () => {
    openEditor();
    fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    fireEvent.click(screen.getByRole("button", { name: "Keep it" }));
    expect(screen.getByLabelText("Service")).toBeTruthy();
    expect(deleteAccount).not.toHaveBeenCalled();
  });
});
