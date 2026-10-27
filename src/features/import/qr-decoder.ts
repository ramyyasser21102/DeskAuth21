import jsQR from "jsqr";

const MAX_IMAGE_SIDE_PX = 1600;
export const NO_QR_FOUND_MESSAGE = "no QR code found";

export interface PixelImage {
  data: Uint8ClampedArray;
  width: number;
  height: number;
}

/** Shrinks large screenshots so decoding stays fast; never enlarges. */
export function scaleToFit(width: number, height: number, maxSide: number): { width: number; height: number } {
  const scale = Math.min(1, maxSide / Math.max(width, height));
  return { width: Math.round(width * scale), height: Math.round(height * scale) };
}

export function findQrText(image: PixelImage): string | null {
  return jsQR(image.data, image.width, image.height)?.data ?? null;
}

function drawToPixels(bitmap: ImageBitmap): PixelImage {
  const { width, height } = scaleToFit(bitmap.width, bitmap.height, MAX_IMAGE_SIDE_PX);
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) throw new Error("canvas is not available");
  context.drawImage(bitmap, 0, 0, width, height);
  return context.getImageData(0, 0, width, height);
}

/** Decodes the QR code in an image file (or pasted screenshot) into its text. */
export async function readQrTextFromImage(file: Blob): Promise<string> {
  const bitmap = await createImageBitmap(file);
  try {
    const text = findQrText(drawToPixels(bitmap));
    if (text === null) throw new Error(NO_QR_FOUND_MESSAGE);
    return text;
  } finally {
    bitmap.close();
  }
}
