import { readBarcodes, type ReaderOptions } from "zxing-wasm/reader";
import { ensureQrDecoderConfigured } from "./qr-wasm";
import { firstQrText, planScales } from "./qr-scales";

export const NO_QR_FOUND_MESSAGE = "no QR code found; try capturing just the code, larger on screen";

export interface PixelImage {
  data: Uint8ClampedArray<ArrayBuffer>;
  width: number;
  height: number;
}

/** Tolerant settings: screenshots can be dark-mode (inverted), tiny, or low contrast. */
const READER_OPTIONS: ReaderOptions = {
  formats: ["QRCode"],
  tryHarder: true,
  tryRotate: true,
  tryInvert: true,
  tryDownscale: true,
};

export async function findQrText(image: PixelImage): Promise<string | null> {
  ensureQrDecoderConfigured();
  const results = await readBarcodes(
    { data: image.data, width: image.width, height: image.height, colorSpace: "srgb" },
    READER_OPTIONS,
  );
  return results.find((result) => result.isValid)?.text ?? null;
}

function drawScaled(bitmap: ImageBitmap, scale: number): PixelImage {
  const width = Math.round(bitmap.width * scale);
  const height = Math.round(bitmap.height * scale);
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) throw new Error("canvas is not available");
  context.drawImage(bitmap, 0, 0, width, height);
  return context.getImageData(0, 0, width, height);
}

/**
 * Decodes the QR code in an image file (or pasted screenshot) into its text.
 * Tries the image as captured, then enlarged, because a dense code shown on a
 * high-resolution screen can have modules only a pixel or two wide.
 */
export async function readQrTextFromImage(file: Blob): Promise<string> {
  const bitmap = await createImageBitmap(file);
  try {
    const scales = planScales(bitmap.width, bitmap.height);
    const text = await firstQrText(scales, (scale) => findQrText(drawScaled(bitmap, scale)));
    if (text === null) throw new Error(NO_QR_FOUND_MESSAGE);
    return text;
  } finally {
    bitmap.close();
  }
}
