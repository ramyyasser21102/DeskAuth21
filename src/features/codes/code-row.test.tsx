import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { copyCode, type CodeView } from "@/lib/api";
import { CodeRow } from "./code-row";

vi.mock("@/lib/api", () => ({ copyCode: vi.fn() }));

const VIEW: CodeView = { label: "Acme (a@b)", code: "123456", seconds_left: 12 };
const THIRD_ACCOUNT_INDEX = 3;
const renderRow = (accountIndex = THIRD_ACCOUNT_INDEX) =>
  render(<ul><CodeRow view={VIEW} accountIndex={accountIndex} /></ul>);

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("CodeRow", () => {
  it("shows the label, code and countdown", () => {
    renderRow();
    expect(screen.getByText("Acme (a@b)")).toBeTruthy();
    expect(screen.getByText("123456")).toBeTruthy();
    expect(screen.getByText("12s")).toBeTruthy();
  });

  it("asks the backend to copy this account and confirms", async () => {
    vi.mocked(copyCode).mockResolvedValue(undefined);
    renderRow();
    fireEvent.click(screen.getByRole("button", { name: "Copy code for Acme (a@b)" }));
    await waitFor(() => expect(copyCode).toHaveBeenCalledWith(THIRD_ACCOUNT_INDEX));
    expect((await screen.findAllByText("Copied")).length).toBeGreaterThan(0);
  });

  it("reports a failed copy", async () => {
    vi.mocked(copyCode).mockRejectedValue("clipboard unavailable");
    renderRow();
    fireEvent.click(screen.getByRole("button"));
    expect((await screen.findAllByText("Copy failed")).length).toBeGreaterThan(0);
  });
});
