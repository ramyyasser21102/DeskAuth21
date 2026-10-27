import { readQrTextFromImage } from "./qr-decoder";

/** Export files are small; refusing huge ones stops a wrong file freezing the window. */
const BYTES_PER_KILOBYTE = 1024;
const KILOBYTES_PER_MEGABYTE = 1024;
const BYTES_PER_MEGABYTE = BYTES_PER_KILOBYTE * KILOBYTES_PER_MEGABYTE;
const MAX_TEXT_FILE_MEGABYTES = 5;
const MAX_TEXT_FILE_BYTES = MAX_TEXT_FILE_MEGABYTES * BYTES_PER_MEGABYTE;

function readAsText(file: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("could not read the file"));
    reader.readAsText(file);
  });
}

/**
 * The text to import from a file: the QR code inside an image, or the contents of
 * an export file (JSON, CSV or a list of otpauth links).
 */
export async function readImportText(file: File): Promise<string> {
  if (file.type.startsWith("image/")) return readQrTextFromImage(file);
  if (file.size > MAX_TEXT_FILE_BYTES) throw new Error("file is too large to be an export");
  return readAsText(file);
}
