import { describe, expect, it, vi } from "vitest";
import { readImportText } from "./read-import-text";

vi.mock("./qr-decoder", () => ({ readQrTextFromImage: vi.fn(async () => "decoded-from-image") }));

const EXPORT_JSON = '{"db":{"entries":[]}}';

describe("readImportText", () => {
  it("decodes the QR code in an image", async () => {
    const image = new File(["pixels"], "qr.png", { type: "image/png" });
    expect(await readImportText(image)).toBe("decoded-from-image");
  });

  it("returns the contents of an export file unchanged", async () => {
    const file = new File([EXPORT_JSON], "aegis.json", { type: "application/json" });
    expect(await readImportText(file)).toBe(EXPORT_JSON);
  });

  it("treats a file with no recognised type as text", async () => {
    const file = new File(["otpauth://totp/a?secret=ABC"], "links");
    expect(await readImportText(file)).toBe("otpauth://totp/a?secret=ABC");
  });

  it("refuses a file too large to be an export", async () => {
    const huge = new File([new Uint8Array(5 * 1024 * 1024 + 1)], "huge.json", { type: "application/json" });
    await expect(readImportText(huge)).rejects.toThrow("too large");
  });
});
