export const PASSPHRASE_MISMATCH_MESSAGE = "Passphrases do not match. Retype the confirmation.";

/** The backend enforces length; this only catches typos in the confirmation. */
export function findPassphraseProblem(
  passphrase: string,
  confirmation: string,
  requiresConfirmation: boolean,
): string | null {
  const isMismatch = requiresConfirmation && passphrase !== confirmation;
  return isMismatch ? PASSPHRASE_MISMATCH_MESSAGE : null;
}
