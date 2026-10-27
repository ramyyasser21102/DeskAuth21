// @vitest-environment node
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { beforeAll, describe, expect, it } from "vitest";
import { findQrText } from "./qr-decoder";
import { configureQrDecoder } from "./qr-wasm";
import { embedInScreenshot, googleExportLink, renderQr, resample } from "./test-support/qr-fixtures";

beforeAll(() => {
  const wasmPath = createRequire(import.meta.url).resolve("zxing-wasm/reader/zxing_reader.wasm");
  const wasm = readFileSync(wasmPath);
  configureQrDecoder({ wasmBinary: wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength) as ArrayBuffer });
});

const FIVE_ACCOUNTS = googleExportLink(5);
const TEN_ACCOUNTS = googleExportLink(10);

describe("findQrText on ordinary images", () => {
  it("decodes a clean QR code", async () => {
    expect(await findQrText(renderQr(FIVE_ACCOUNTS, { modulePx: 6 }))).toBe(FIVE_ACCOUNTS);
  });

  it("returns null for an image without a QR code", async () => {
    const blank = { data: new Uint8ClampedArray(64 * 64 * 4).fill(255), width: 64, height: 64 };
    expect(await findQrText(blank)).toBeNull();
  });

  it("decodes a dark-mode code (light modules on a dark background)", async () => {
    const inverted = renderQr(FIVE_ACCOUNTS, { modulePx: 4, dark: [255, 255, 255], light: [0, 0, 0] });
    expect(await findQrText(inverted)).toBe(FIVE_ACCOUNTS);
  });

  it("decodes a low-contrast grey-on-grey code", async () => {
    const faint = renderQr(FIVE_ACCOUNTS, { modulePx: 4, dark: [120, 120, 120], light: [190, 190, 190] });
    expect(await findQrText(faint)).toBe(FIVE_ACCOUNTS);
  });

  it("decodes a tightly cropped code with no blank border", async () => {
    expect(await findQrText(renderQr(FIVE_ACCOUNTS, { modulePx: 5, quietModules: 0 }))).toBe(FIVE_ACCOUNTS);
  });
});

// These are the cases the previous decoder (jsQR, after shrinking to 1600 px) missed.
describe("findQrText on dense codes in large screenshots", () => {
  it("decodes a 10-account export in a Retina-sized screenshot", async () => {
    const screenshot = embedInScreenshot(renderQr(TEN_ACCOUNTS, { modulePx: 4 }), 2880, 1800, { x: 1500, y: 500 });
    expect(await findQrText(screenshot)).toBe(TEN_ACCOUNTS);
  });

  it("decodes a 10-account export in a 4K screenshot", async () => {
    const screenshot = embedInScreenshot(renderQr(TEN_ACCOUNTS, { modulePx: 3 }), 3840, 2160, { x: 2000, y: 700 });
    expect(await findQrText(screenshot)).toBe(TEN_ACCOUNTS);
  });
});

describe("findQrText on enlarged copies", () => {
  it("decodes a dense code displayed at 2.3 px per module once it is enlarged, as the retry does", async () => {
    const squashed = resample(renderQr(TEN_ACCOUNTS, { modulePx: 3 }), 2.3 / 3);
    expect(await findQrText(resample(squashed, 2))).toBe(TEN_ACCOUNTS);
  });
});
