/** Who the user is on this device. Never part of the vault and never synced. */
export interface Profile {
  name: string;
  /** A small square image as a data URL, or `null` to show initials. */
  pictureDataUrl: string | null;
}

export const EMPTY_PROFILE: Profile = { name: "", pictureDataUrl: null };
export const MAX_NAME_LENGTH = 40;
export const PICTURE_SIDE_PX = 128;
const IMAGE_DATA_URL_PREFIX = "data:image/";
const MAX_INITIALS = 2;
const HALF = 2;

export interface SquareCrop {
  sourceX: number;
  sourceY: number;
  side: number;
}

const isImageDataUrl = (value: unknown): value is string =>
  typeof value === "string" && value.startsWith(IMAGE_DATA_URL_PREFIX);

/** Reads what was saved earlier; anything unexpected quietly becomes the empty profile. */
export function parseProfile(raw: string | null): Profile {
  if (raw === null) return EMPTY_PROFILE;
  try {
    const parsed: unknown = JSON.parse(raw);
    const { name, pictureDataUrl } = (parsed ?? {}) as Record<string, unknown>;
    return {
      name: typeof name === "string" ? name.slice(0, MAX_NAME_LENGTH) : "",
      pictureDataUrl: isImageDataUrl(pictureDataUrl) ? pictureDataUrl : null,
    };
  } catch {
    return EMPTY_PROFILE;
  }
}

export const serializeProfile = (profile: Profile): string => JSON.stringify(profile);

/** "Ada Lovelace" becomes "AL"; one word gives one letter; blank gives nothing. */
export function initialsOf(name: string): string {
  return name
    .trim()
    .split(/\s+/)
    .filter((word) => word !== "")
    .slice(0, MAX_INITIALS)
    .map((word) => [...word][0].toUpperCase())
    .join("");
}

/** The largest centred square inside an image, so any photo becomes a square avatar. */
export function centredSquare(width: number, height: number): SquareCrop {
  const side = Math.min(width, height);
  return { sourceX: Math.floor((width - side) / HALF), sourceY: Math.floor((height - side) / HALF), side };
}
