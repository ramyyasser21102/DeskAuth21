/** Sizes to try, smallest first: as captured, then enlarged for codes drawn with tiny modules. */
const AS_CAPTURED = 1;
const DOUBLED = 2;
const TRIPLED = 3;
const UPSCALE_FACTORS = [AS_CAPTURED, DOUBLED, TRIPLED];
/** Very large images are shrunk first so memory use stays sane. */
const MAX_SIDE_PX = 6000;
/** Enlarging stops once the decoded image would pass this many pixels. */
const MAX_PIXELS = 40_000_000;

/**
 * The scales at which to attempt decoding an image of this size. Never empty:
 * the first attempt is always the image itself (shrunk only if enormous).
 */
export function planScales(width: number, height: number): number[] {
  const baseScale = Math.min(1, MAX_SIDE_PX / Math.max(width, height));
  const scales = UPSCALE_FACTORS.map((factor) => factor * baseScale);
  const affordable = scales.filter((scale) => width * scale * height * scale <= MAX_PIXELS);
  return affordable.length > 0 ? affordable : [baseScale];
}

/** The first non-null result of `decodeAt`, trying each scale in order. */
export async function firstQrText(
  scales: readonly number[],
  decodeAt: (scale: number) => Promise<string | null>,
): Promise<string | null> {
  for (const scale of scales) {
    const text = await decodeAt(scale);
    if (text !== null) return text;
  }
  return null;
}
