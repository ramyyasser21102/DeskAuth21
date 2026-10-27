import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { importUri } from "@/lib/api";
import { ImportPanel } from "./import-panel";
import { readQrTextFromImage } from "./qr-decoder";

vi.mock("@/lib/api", () => ({ importUri: vi.fn() }));
vi.mock("./qr-decoder", () => ({ readQrTextFromImage: vi.fn() }));

const LINK = "otpauth://totp/Acme:a@b?secret=GEZDGNBVGY3TQOJQ";
const png = (name: string) => new File(["pixels"], name, { type: "image/png" });

beforeEach(() => {
  vi.mocked(readQrTextFromImage).mockImplementation(async (file) => `qr-from-${(file as File).name}`);
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const chooseImages = (...files: File[]) =>
  fireEvent.change(screen.getByLabelText("QR code images"), { target: { files } });

describe("ImportPanel with a pasted link", () => {
  it("imports the trimmed link, reports the result and clears the field", async () => {
    vi.mocked(importUri).mockResolvedValue({ added: 2, skipped_duplicates: 0 });
    render(<ImportPanel />);
    const field = screen.getByLabelText("Paste an otpauth:// link") as HTMLInputElement;
    fireEvent.change(field, { target: { value: `  ${LINK}  ` } });
    fireEvent.click(screen.getByRole("button", { name: "Import" }));
    expect(await screen.findByText("Added 2 accounts.")).toBeTruthy();
    expect(importUri).toHaveBeenCalledWith(LINK);
    expect(field.value).toBe("");
  });

  it("shows the backend's error message", async () => {
    vi.mocked(importUri).mockRejectedValue("unsupported or malformed URI");
    render(<ImportPanel />);
    fireEvent.change(screen.getByLabelText("Paste an otpauth:// link"), { target: { value: "nope" } });
    fireEvent.click(screen.getByRole("button", { name: "Import" }));
    expect(await screen.findByText("unsupported or malformed URI")).toBeTruthy();
  });

  it("keeps Import disabled until something is typed", () => {
    render(<ImportPanel />);
    expect((screen.getByRole("button", { name: "Import" }) as HTMLButtonElement).disabled).toBe(true);
  });
});

describe("ImportPanel with chosen images", () => {
  it("imports every chosen image and combines the results", async () => {
    vi.mocked(importUri).mockResolvedValue({ added: 1, skipped_duplicates: 0 });
    render(<ImportPanel />);
    chooseImages(png("first.png"), png("second.png"));
    expect(await screen.findByText("Added 2 accounts.")).toBeTruthy();
    expect(importUri).toHaveBeenCalledWith("qr-from-first.png");
    expect(importUri).toHaveBeenCalledWith("qr-from-second.png");
  });

  it("still imports the good images when one has no QR code", async () => {
    vi.mocked(importUri).mockResolvedValue({ added: 1, skipped_duplicates: 0 });
    vi.mocked(readQrTextFromImage).mockImplementation(async (file) => {
      if ((file as File).name === "bad.png") throw new Error("no QR code found");
      return "good-link";
    });
    render(<ImportPanel />);
    chooseImages(png("good.png"), png("bad.png"));
    expect(await screen.findByText("Added 1 account. Could not import bad.png (no QR code found).")).toBeTruthy();
  });

});

describe("ImportPanel with pasted images", () => {
  it("imports an image pasted from the clipboard", async () => {
    vi.mocked(importUri).mockResolvedValue({ added: 1, skipped_duplicates: 1 });
    render(<ImportPanel />);
    fireEvent.paste(document, { clipboardData: { files: [png("screenshot.png")] } });
    await waitFor(() => expect(importUri).toHaveBeenCalledWith("qr-from-screenshot.png"));
    expect(await screen.findByText("Added 1 account. Skipped 1 already in your vault.")).toBeTruthy();
  });

  it("ignores pasted text and non-image files", () => {
    render(<ImportPanel />);
    fireEvent.paste(document, { clipboardData: { files: [new File(["x"], "notes.txt", { type: "text/plain" })] } });
    expect(importUri).not.toHaveBeenCalled();
  });
});
