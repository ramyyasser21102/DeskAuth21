/** The times offered for clearing a copied code; the backend accepts exactly these. */
const TEN_SECONDS = 10;
const FIFTEEN_SECONDS = 15;
const THIRTY_SECONDS = 30;
const SIXTY_SECONDS = 60;
export const CLIPBOARD_CLEAR_OPTIONS: readonly number[] = [TEN_SECONDS, FIFTEEN_SECONDS, THIRTY_SECONDS, SIXTY_SECONDS];

export const describeClipboardClearing = (seconds: number): string =>
  `A copied code is removed from the clipboard after ${seconds} seconds, when the vault locks and when you quit, if the clipboard still holds it.`;
