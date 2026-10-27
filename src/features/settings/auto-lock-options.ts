export interface AutoLockOption {
  minutes: number | null;
  label: string;
}

/** Must match ALLOWED_AUTO_LOCK_MINUTES in the Rust backend, which validates them. */
export const AUTO_LOCK_OPTIONS: readonly AutoLockOption[] = [
  { minutes: 1, label: "1 min" },
  { minutes: 5, label: "5 min" },
  { minutes: 15, label: "15 min" },
  { minutes: 30, label: "30 min" },
  { minutes: null, label: "Never" },
];

export function describeAutoLock(minutes: number | null): string {
  if (minutes === null) return "The vault stays unlocked until you lock it.";
  return `The vault locks after ${minutes} ${minutes === 1 ? "minute" : "minutes"} without activity.`;
}
