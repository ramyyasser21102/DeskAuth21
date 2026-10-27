export const RECOVERY_WORD_COUNT = 24;
/** Enough to prove the person wrote the words down, without retyping all of them. */
export const CONFIRMATION_WORD_COUNT = 3;

export const splitWords = (phrase: string): string[] => phrase.trim().split(/\s+/).filter((word) => word !== "");

/**
 * Which words to ask for, as distinct positions in ascending order. `pickIndex(limit)`
 * returns a random whole number below `limit`; it is passed in so tests can control it.
 */
export function pickConfirmationPositions(wordCount: number, howMany: number, pickIndex: (limit: number) => number): number[] {
  const chosen = new Set<number>();
  while (chosen.size < howMany) chosen.add(pickIndex(wordCount));
  return [...chosen].sort((first, second) => first - second);
}

/** Case and surrounding spaces do not matter when retyping. */
export function confirmationMatches(words: readonly string[], positions: readonly number[], typed: readonly string[]): boolean {
  return positions.every((position, index) => typed[index]?.trim().toLowerCase() === words[position]);
}

export const randomIndexBelow = (limit: number): number => crypto.getRandomValues(new Uint32Array(1))[0] % limit;
