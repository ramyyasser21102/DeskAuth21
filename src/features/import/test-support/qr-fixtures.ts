// Builds realistic QR images for tests: dense Google exports, shown the way real
// screenshots show them. Not used by the app itself.
import QRCode from "qrcode";

export interface Rgba {
  data: Uint8ClampedArray<ArrayBuffer>;
  width: number;
  height: number;
}

type Color = readonly [number, number, number];
const BLACK: Color = [0, 0, 0];
const WHITE: Color = [255, 255, 255];
const OPAQUE = 255;
const CHANNELS = 4;

const varint = (value: number): number[] => {
  const bytes: number[] = [];
  let rest = value;
  while (rest > 0x7f) {
    bytes.push((rest & 0x7f) | 0x80);
    rest >>>= 7;
  }
  return [...bytes, rest];
};
const text = (value: string): number[] => [...new TextEncoder().encode(value)];
const lengthDelimited = (field: number, body: number[]): number[] => [...varint((field << 3) | 2), ...varint(body.length), ...body];
const varintField = (field: number, value: number): number[] => [...varint(field << 3), ...varint(value)];

function otpParameters(index: number): number[] {
  return [
    ...lengthDelimited(1, text(`TEST-ONLY-FAKE-SECRET-${String(index).padStart(4, "0")}`)),
    ...lengthDelimited(2, text(`Service${index}:user${index}@example.com`)),
    ...lengthDelimited(3, text(`Service${index}`)),
    ...varintField(4, 1),
    ...varintField(5, 1),
    ...varintField(6, 2),
  ];
}

/** A Google Authenticator export link holding that many made-up accounts. */
export function googleExportLink(accountCount: number): string {
  const accounts = Array.from({ length: accountCount }, (_unused, index) => lengthDelimited(1, otpParameters(index)));
  const payload = [...accounts.flat(), ...varintField(2, 1), ...varintField(3, 1), ...varintField(4, 0)];
  return `otpauth-migration://offline?data=${encodeURIComponent(btoa(String.fromCharCode(...payload)))}`;
}

function filled(width: number, height: number, color: Color): Rgba {
  const data = new Uint8ClampedArray(width * height * CHANNELS);
  for (let pixel = 0; pixel < width * height; pixel++) data.set([...color, OPAQUE], pixel * CHANNELS);
  return { data, width, height };
}

interface QrStyle {
  modulePx: number;
  dark?: Color;
  light?: Color;
  quietModules?: number;
}

function isDarkModule(modules: Uint8Array, size: number, column: number, row: number): boolean {
  const isInside = column >= 0 && row >= 0 && column < size && row < size;
  return isInside && modules[row * size + column] === 1;
}

/** The QR drawn with `modulePx` pixels per module, plus the blank border scanners expect. */
export function renderQr(link: string, style: QrStyle): Rgba {
  const { modulePx, dark = BLACK, light = WHITE, quietModules = 4 } = style;
  const { size, data: modules } = QRCode.create(link, { errorCorrectionLevel: "L" }).modules;
  const side = (size + quietModules * 2) * modulePx;
  const image = filled(side, side, light);
  for (let y = 0; y < side; y++) {
    for (let x = 0; x < side; x++) {
      const column = Math.floor(x / modulePx) - quietModules;
      const row = Math.floor(y / modulePx) - quietModules;
      if (isDarkModule(modules, size, column, row)) image.data.set([...dark, OPAQUE], (y * side + x) * CHANNELS);
    }
  }
  return image;
}

/** A full-screen capture with the QR somewhere on a plain background, as in a real screenshot. */
export function embedInScreenshot(qr: Rgba, width: number, height: number, at: { x: number; y: number }): Rgba {
  const screen = filled(width, height, [236, 238, 242]);
  for (let y = 0; y < qr.height; y++) {
    const from = y * qr.width * CHANNELS;
    screen.data.set(qr.data.subarray(from, from + qr.width * CHANNELS), ((at.y + y) * width + at.x) * CHANNELS);
  }
  return screen;
}

/** Bilinear resize, which is what a display scaled to a non-integer factor does to a code. */
export function resample(image: Rgba, scale: number): Rgba {
  const width = Math.round(image.width * scale);
  const height = Math.round(image.height * scale);
  const out = filled(width, height, BLACK);
  const at = (x: number, y: number, channel: number) => image.data[(y * image.width + x) * CHANNELS + channel];
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const sourceX = Math.min(image.width - 1, (x + 0.5) / scale - 0.5);
      const sourceY = Math.min(image.height - 1, (y + 0.5) / scale - 0.5);
      const x0 = Math.max(0, Math.floor(sourceX));
      const y0 = Math.max(0, Math.floor(sourceY));
      const x1 = Math.min(image.width - 1, x0 + 1);
      const y1 = Math.min(image.height - 1, y0 + 1);
      const [fx, fy] = [sourceX - x0, sourceY - y0];
      for (let channel = 0; channel < 3; channel++) {
        const top = at(x0, y0, channel) * (1 - fx) + at(x1, y0, channel) * fx;
        const bottom = at(x0, y1, channel) * (1 - fx) + at(x1, y1, channel) * fx;
        out.data[(y * width + x) * CHANNELS + channel] = top * (1 - fy) + bottom * fy;
      }
    }
  }
  return out;
}
