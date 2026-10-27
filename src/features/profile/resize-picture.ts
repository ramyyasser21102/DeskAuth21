import { centredSquare, PICTURE_SIDE_PX } from "./profile";

const PICTURE_MIME_TYPE = "image/jpeg";
const PICTURE_QUALITY = 0.85;

/** Shrinks any chosen image to a small square so the stored avatar stays tiny. */
export async function resizeToAvatar(file: File): Promise<string> {
  const bitmap = await createImageBitmap(file);
  try {
    const { sourceX, sourceY, side } = centredSquare(bitmap.width, bitmap.height);
    const canvas = document.createElement("canvas");
    canvas.width = PICTURE_SIDE_PX;
    canvas.height = PICTURE_SIDE_PX;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("this window cannot resize images");
    context.drawImage(bitmap, sourceX, sourceY, side, side, 0, 0, PICTURE_SIDE_PX, PICTURE_SIDE_PX);
    return canvas.toDataURL(PICTURE_MIME_TYPE, PICTURE_QUALITY);
  } finally {
    bitmap.close();
  }
}
