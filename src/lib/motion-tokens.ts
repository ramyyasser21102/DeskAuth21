import type { Transition } from "motion/react";

/** Seconds. Nothing in the app animates for any other length. */
export const DURATION = { quick: 0.1, fast: 0.15, base: 0.2, slow: 0.3 } as const;

/** Cubic-bezier control points: x1, y1, x2, y2. */
type Curve = readonly [number, number, number, number];

const FAST_START = 0.16;
const SOFT_LANDING = 0.3;
const SLOW_START = 0.7;
const FAST_FINISH = 0.84;

const EASE_OUT_CURVE: Curve = [FAST_START, 1, SOFT_LANDING, 1];
const EASE_IN_CURVE: Curve = [SLOW_START, 0, FAST_FINISH, 0];

/** Things arriving slow down into place; things leaving speed up and go. */
export const ENTER: Transition = { duration: DURATION.base, ease: EASE_OUT_CURVE };
export const EXIT: Transition = { duration: DURATION.fast, ease: EASE_IN_CURVE };

const PAGE_RISE_PIXELS = 8;
const ROW_RISE_PIXELS = 4;

/** Only opacity and transform change, so nothing reflows while it plays. */
export const PAGE_MOTION = {
  initial: { opacity: 0, y: PAGE_RISE_PIXELS },
  animate: { opacity: 1, y: 0, transition: ENTER },
  exit: { opacity: 0, transition: EXIT },
} as const;

export const ROW_MOTION = {
  initial: { opacity: 0, y: ROW_RISE_PIXELS },
  animate: { opacity: 1, y: 0, transition: ENTER },
  exit: { opacity: 0, transition: EXIT },
} as const;

const STAGGER_SECONDS = 0.05;
const TAP_SCALE = 0.98;

/** A parent that reveals its `REVEAL_ITEM` children one after another. */
export const REVEAL_GROUP = {
  hidden: {},
  show: { transition: { staggerChildren: STAGGER_SECONDS } },
} as const;

export const REVEAL_ITEM = {
  hidden: { opacity: 0, y: PAGE_RISE_PIXELS },
  show: { opacity: 1, y: 0, transition: ENTER },
} as const;

/** A press sinks the control a hair; it is never a bounce. */
export const TAP_MOTION = { scale: TAP_SCALE, transition: { duration: DURATION.quick } } as const;

/** A small swap, such as a number ticking over or a status replacing a ring. */
export const SWAP_MOTION = {
  initial: { opacity: 0, y: ROW_RISE_PIXELS },
  animate: { opacity: 1, y: 0, transition: { duration: DURATION.quick, ease: EASE_OUT_CURVE } },
  exit: { opacity: 0, transition: { duration: DURATION.quick, ease: EASE_IN_CURVE } },
} as const;
