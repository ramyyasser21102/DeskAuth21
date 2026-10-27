import { AnimatePresence, motion, useReducedMotion, type Transition } from "motion/react";
import { SWAP_MOTION } from "@/lib/motion-tokens";
import { cn } from "@/lib/utils";
import { useRingSweep } from "./countdown-motion";
import { dashOffsetFor, fractionRemaining, isExpiringSoon } from "./countdown";

const RING_SIZE = 36;
const STROKE_WIDTH = 3;
const HALF = 2;
const RING_CENTRE = RING_SIZE / HALF;
const RING_DIAMETER = RING_SIZE - STROKE_WIDTH;
const RING_RADIUS = RING_DIAMETER / HALF;
const CIRCUMFERENCE = Math.PI * RING_DIAMETER;

function RingSvg({ offset, sweep }: { offset: number; sweep: Transition }) {
  return (
    <svg width={RING_SIZE} height={RING_SIZE} className="-rotate-90" aria-hidden="true">
        <circle cx={RING_CENTRE} cy={RING_CENTRE} r={RING_RADIUS} fill="none" strokeWidth={STROKE_WIDTH} className="stroke-border" />
        <motion.circle
          cx={RING_CENTRE}
          cy={RING_CENTRE}
          r={RING_RADIUS}
          fill="none"
          strokeWidth={STROKE_WIDTH}
          strokeLinecap="round"
          strokeDasharray={CIRCUMFERENCE}
          initial={false}
          animate={{ strokeDashoffset: offset }}
          transition={sweep}
          className="stroke-foreground"
        />
      </svg>
  );
}

interface CountdownRingProps {
  secondsLeft: number;
  periodSeconds: number;
}

/**
 * A ring that empties as the code runs out, with the seconds inside. It drains
 * in one steady sweep and refills quickly when a new code arrives.
 */
export function CountdownRing({ secondsLeft, periodSeconds }: CountdownRingProps) {
  const fraction = fractionRemaining(secondsLeft, periodSeconds);
  const offset = dashOffsetFor(CIRCUMFERENCE, fraction);
  const sweep = useRingSweep(fraction, useReducedMotion() ?? false);
  const isUrgent = isExpiringSoon(secondsLeft);
  return (
    <span
      role="img"
      aria-label={`${secondsLeft} seconds left`}
      className={cn("relative inline-flex shrink-0 items-center justify-center", isUrgent && "animate-pulse motion-reduce:animate-none")}
      style={{ width: RING_SIZE, height: RING_SIZE }}
    >
      <RingSvg offset={offset} sweep={sweep} />
      <AnimatePresence mode="popLayout" initial={false}>
        <motion.span key={secondsLeft} aria-hidden="true" className="absolute text-xs font-medium tabular-nums" {...SWAP_MOTION}>{secondsLeft}</motion.span>
      </AnimatePresence>
    </span>
  );
}
