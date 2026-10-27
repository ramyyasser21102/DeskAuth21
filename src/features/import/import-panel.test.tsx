import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { importText } from "@/lib/api";
import { ImportPanel } from "./import-panel";
import { readQrTextFromImage } from "./qr-decoder";

vi.mock("@/lib/api", () => ({ importText: vi.fn() }));
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
  fireEvent.change(screen.getByLabelText("Files to import"), { target: { files } });

describe("ImportPanel with a pasted link", () => {
  it("imports the trimmed link, reports the result and clears the field", async () => {
    vi.mocked(importText).mockResolvedValue({ added: 2, skipped_duplicates: 0, skipped_unsupported: 0 });
    render(<ImportPanel vendor="other" />);
    const field = screen.getByLabelText("Paste an otpauth:// link") as HTMLInputElement;
    fireEvent.change(field, { target: { value: `  ${LINK}  ` } });
    fireEvent.click(screen.getByRole("button", { name: "Import" }));
    expect(await screen.findByText("Added 2 accounts.")).toBeTruthy();
    expect(importText).toHaveBeenCalledWith(LINK, "other");
    expect(field.value).toBe("");
  });

  it("shows the backend's error message", async () => {
    vi.mocked(importText).mockRejectedValue("unsupported or malformed URI");
    render(<ImportPanel vendor="other" />);
    fireEvent.change(screen.getByLabelText("Paste an otpauth:// link"), { target: { value: "nope" } });
    fireEvent.click(screen.getByRole("button", { name: "Import" }));
    expect(await screen.findByText("unsupported or malformed URI")).toBeTruthy();
  });

  it("keeps Import disabled until something is typed", () => {
    render(<ImportPanel vendor="other" />);
    expect((screen.getByRole("button", { name: "Import" }) as HTMLButtonElement).disabled).toBe(true);
  });
});

describe("ImportPanel with chosen images", () => {
  it("imports every chosen image and combines the results", async () => {
    vi.mocked(importText).mockResolvedValue({ added: 1, skipped_duplicates: 0, skipped_unsupported: 0 });
    render(<ImportPanel vendor="other" />);
    chooseImages(png("first.png"), png("second.png"));
    expect(await screen.findByText("Added 2 accounts.")).toBeTruthy();
    expect(importText).toHaveBeenCalledWith("qr-from-first.png", "other");
    expect(importText).toHaveBeenCalledWith("qr-from-second.png", "other");
  });

  it("still imports the good images when one has no QR code", async () => {
    vi.mocked(importText).mockResolvedValue({ added: 1, skipped_duplicates: 0, skipped_unsupported: 0 });
    vi.mocked(readQrTextFromImage).mockImplementation(async (file) => {
      if ((file as File).name === "bad.png") throw new Error("no QR code found");
      return "good-link";
    });
    render(<ImportPanel vendor="other" />);
    chooseImages(png("good.png"), png("bad.png"));
    expect(await screen.findByText("Added 1 account. Could not import bad.png (no QR code found).")).toBeTruthy();
  });

});

describe("ImportPanel vendor tag", () => {
  it("passes the chosen app along with the link", async () => {
    vi.mocked(importText).mockResolvedValue({ added: 1, skipped_duplicates: 0, skipped_unsupported: 0 });
    render(<ImportPanel vendor="microsoft_authenticator" />);
    fireEvent.change(screen.getByLabelText("Paste an otpauth:// link"), { target: { value: LINK } });
    fireEvent.click(screen.getByRole("button", { name: "Import" }));
    await waitFor(() => expect(importText).toHaveBeenCalledWith(LINK, "microsoft_authenticator"));
  });
});

describe("ImportPanel with pasted images", () => {
  it("imports an image pasted from the clipboard", async () => {
    vi.mocked(importText).mockResolvedValue({ added: 1, skipped_duplicates: 1, skipped_unsupported: 0 });
    render(<ImportPanel vendor="other" />);
    fireEvent.paste(document, { clipboardData: { files: [png("screenshot.png")] } });
    await waitFor(() => expect(importText).toHaveBeenCalledWith("qr-from-screenshot.png", "other"));
    expect(await screen.findByText("Added 1 account. Skipped 1 already in your vault.")).toBeTruthy();
  });

  it("ignores pasted text and non-image files", () => {
    render(<ImportPanel vendor="other" />);
    fireEvent.paste(document, { clipboardData: { files: [new File(["x"], "notes.txt", { type: "text/plain" })] } });
    expect(importText).not.toHaveBeenCalled();
  });
});

describe("ImportPanel with export files", () => {
  it("sends the contents of an export file to the backend as it is", async () => {
    vi.mocked(importText).mockResolvedValue({ added: 3, skipped_duplicates: 0, skipped_unsupported: 1 });
    render(<ImportPanel vendor="other" />);
    const contents = '{"db":{"entries":[]}}';
    chooseImages(new File([contents], "aegis.json", { type: "application/json" }));
    expect(await screen.findByText(/Added 3 accounts\. 1 entry could not be imported/)).toBeTruthy();
    expect(importText).toHaveBeenCalledWith(contents, "other");
    expect(readQrTextFromImage).not.toHaveBeenCalled();
  });

  it("explains an encrypted export the backend refuses", async () => {
    vi.mocked(importText).mockRejectedValue("this Aegis export is password-protected; export it again without a password");
    render(<ImportPanel vendor="other" />);
    chooseImages(new File(["{}"], "aegis.json", { type: "application/json" }));
    expect(await screen.findByText(/Could not import aegis\.json .*password-protected/)).toBeTruthy();
  });
});
