export const PASSPHRASE_MISMATCH_MESSAGE = "Passphrases do not match. Retype the confirmation.";

/** The same minimum the backend enforces for new vaults; existing vaults may be shorter. */
export const MIN_PASSPHRASE_CHARS = 12;
export const PASSPHRASE_TOO_SHORT_MESSAGE = `Use at least ${MIN_PASSPHRASE_CHARS} characters, or generate a passphrase.`;

/** Counts characters, not UTF-16 units, so an emoji is one. */
export const countCharacters = (text: string): number => [...text].length;

/**
 * Only choosing a new passphrase is checked here (the backend enforces it too);
 * unlocking must accept whatever an existing vault was made with.
 */
export function findPassphraseProblem(
  passphrase: string,
  confirmation: string,
  isChoosingNew: boolean,
): string | null {
  if (!isChoosingNew) return null;
  if (countCharacters(passphrase) < MIN_PASSPHRASE_CHARS) return PASSPHRASE_TOO_SHORT_MESSAGE;
  return passphrase === confirmation ? null : PASSPHRASE_MISMATCH_MESSAGE;
}
