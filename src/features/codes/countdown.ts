/** From this many seconds left the ring starts pulsing to say the code is about to change. */
export const EXPIRING_SOON_SECONDS = 5;

/** How much of the code's window is left, from 0 (just expired) to 1 (just issued). */
export function fractionRemaining(secondsLeft: number, periodSeconds: number): number {
  if (periodSeconds <= 0) return 0;
  return Math.min(1, Math.max(0, secondsLeft / periodSeconds));
}

export const isExpiringSoon = (secondsLeft: number): boolean => secondsLeft <= EXPIRING_SOON_SECONDS;

/** Stroke offset that leaves `fraction` of a circle's outline drawn. */
export const dashOffsetFor = (circumference: number, fraction: number): number => circumference * (1 - fraction);
