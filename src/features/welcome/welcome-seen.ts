import { browserStorage } from "@/features/theme/theme-preference";

export const WELCOME_SEEN_KEY = "deskauth21.welcome-seen";
const SEEN_VALUE = "yes";

/** If storage is unavailable the welcome page simply shows again next time. */
export function hasSeenWelcome(): boolean {
  try {
    return browserStorage.getItem(WELCOME_SEEN_KEY) === SEEN_VALUE;
  } catch {
    return false;
  }
}

export function markWelcomeSeen(): void {
  try {
    browserStorage.setItem(WELCOME_SEEN_KEY, SEEN_VALUE);
  } catch {
    // Seeing the welcome page again is harmless.
  }
}
