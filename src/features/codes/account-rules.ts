export const EMPTY_NAME_MESSAGE = "An account needs a name.";

/** The backend enforces this too; checking here just avoids a round trip. */
export function findNameProblem(name: string): string | null {
  return name.trim() === "" ? EMPTY_NAME_MESSAGE : null;
}
