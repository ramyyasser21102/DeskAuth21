import type { RelocationOutcome } from "@/lib/api";

const MESSAGES: Record<RelocationOutcome, string> = {
  unchanged: "That is already where your vault is.",
  moved:
    "Vault moved. On your other devices, install DeskAuth21, choose this same folder and unlock with the same passphrase.",
  used_existing: "Now using the vault in that folder. Unlock it with its passphrase.",
  selected: "Folder chosen. Your vault will be created there.",
};

export const describeRelocation = (outcome: RelocationOutcome): string => MESSAGES[outcome];

/** After these the vault on screen is no longer the one that was unlocked. */
export const requiresUnlockAgain = (outcome: RelocationOutcome): boolean =>
  outcome === "used_existing" || outcome === "selected";
