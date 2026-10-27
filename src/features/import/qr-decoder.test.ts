import QRCode from "qrcode";
import { describe, expect, it } from "vitest";
import { findQrText, scaleToFit, type PixelImage } from "./qr-decoder";

const MODULE_SIZE_PX = 6;
const QUIET_ZONE_MODULES = 4;
const OPAQUE = 255;

/** Renders a real QR code into raw RGBA pixels, with the white border scanners need. */
function renderQr(text: string): PixelImage {
  const { size, data } = QRCode.create(text).modules;
  const side = (size + QUIET_ZONE_MODULES * 2) * MODULE_SIZE_PX;
  const pixels = new Uint8ClampedArray(side * side * 4).fill(OPAQUE);
  for (let y = 0; y < side; y++) {
    for (let x = 0; x < side; x++) {
      const column = Math.floor(x / MODULE_SIZE_PX) - QUIET_ZONE_MODULES;
      const row = Math.floor(y / MODULE_SIZE_PX) - QUIET_ZONE_MODULES;
      const isDark = column >= 0 && row >= 0 && column < size && row < size && data[row * size + column];
      if (isDark) pixels.fill(0, (y * side + x) * 4, (y * side + x) * 4 + 3);
    }
  }
  return { data: pixels, width: side, height: side };
}

describe("findQrText", () => {
  it("decodes a QR code containing a Google Authenticator export link", () => {
    const link = "otpauth-migration://offline?data=CjEKCkhlbGxvId6tvu8SGEV4YW1wbGU6YWxpY2VAZ21haWwuY29tGgdFeGFtcGxlIAEoATACEAEYASAA";
    expect(findQrText(renderQr(link))).toBe(link);
  });

  it("decodes a plain otpauth link", () => {
    const link = "otpauth://totp/Acme:a@b?secret=GEZDGNBVGY3TQOJQ&issuer=Acme";
    expect(findQrText(renderQr(link))).toBe(link);
  });

  it("returns null for an image without a QR code", () => {
    const blank = { data: new Uint8ClampedArray(64 * 64 * 4).fill(OPAQUE), width: 64, height: 64 };
    expect(findQrText(blank)).toBeNull();
  });
});

describe("scaleToFit", () => {
  it("shrinks large images and preserves aspect ratio", () => {
    expect(scaleToFit(3200, 1600, 1600)).toEqual({ width: 1600, height: 800 });
  });

  it("never enlarges small images", () => {
    expect(scaleToFit(300, 200, 1600)).toEqual({ width: 300, height: 200 });
  });
});
