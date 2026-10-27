import { useState } from "react";
import type { Transition } from "motion/react";
import { DURATION, ENTER } from "@/lib/motion-tokens";

/** The ring drains over a whole second per tick so the ticks read as one continuous sweep. */
const DRAIN_SWEEP: Transition = { duration: 1, ease: "linear" };
const REFILL_SWEEP: Transition = { ...ENTER, duration: DURATION.slow };
const NO_MOTION: Transition = { duration: 0 };

/**
 * Draining is slow and steady; a new code refills the ring quickly instead of
 * running the sweep backwards for a second.
 */
export function ringSweep(previousFraction: number, fraction: number, isMotionReduced: boolean): Transition {
  if (isMotionReduced) return NO_MOTION;
  return fraction > previousFraction ? REFILL_SWEEP : DRAIN_SWEEP;
}

/** The sweep for the ring's latest fraction, remembering the one before it. */
export function useRingSweep(fraction: number, isMotionReduced: boolean): Transition {
  const [seen, setSeen] = useState({ latest: fraction, before: fraction });
  if (seen.latest !== fraction) setSeen({ latest: fraction, before: seen.latest });
  return ringSweep(seen.before, seen.latest === fraction ? fraction : seen.latest, isMotionReduced);
}
