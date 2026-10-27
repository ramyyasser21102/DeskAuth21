import { cn } from "@/lib/utils";
import { dashOffsetFor, fractionRemaining, isExpiringSoon } from "./countdown";

const RING_SIZE = 28;
const STROKE_WIDTH = 3;
const HALF = 2;
const RING_CENTRE = RING_SIZE / HALF;
const RING_DIAMETER = RING_SIZE - STROKE_WIDTH;
const RING_RADIUS = RING_DIAMETER / HALF;
const CIRCUMFERENCE = Math.PI * RING_DIAMETER;

interface CountdownRingProps {
  secondsLeft: number;
  periodSeconds: number;
}

/**
 * A ring that empties as the code runs out, with the seconds inside. The
 * one-second linear transition makes the ticks read as continuous motion.
 */
export function CountdownRing({ secondsLeft, periodSeconds }: CountdownRingProps) {
  const offset = dashOffsetFor(CIRCUMFERENCE, fractionRemaining(secondsLeft, periodSeconds));
  const isUrgent = isExpiringSoon(secondsLeft);
  return (
    <span
      role="img"
      aria-label={`${secondsLeft} seconds left`}
      className={cn("relative inline-flex shrink-0 items-center justify-center", isUrgent && "animate-pulse motion-reduce:animate-none")}
      style={{ width: RING_SIZE, height: RING_SIZE }}
    >
      <svg width={RING_SIZE} height={RING_SIZE} className="-rotate-90" aria-hidden="true">
        <circle cx={RING_CENTRE} cy={RING_CENTRE} r={RING_RADIUS} fill="none" strokeWidth={STROKE_WIDTH} className="stroke-border" />
        <circle
          cx={RING_CENTRE}
          cy={RING_CENTRE}
          r={RING_RADIUS}
          fill="none"
          strokeWidth={STROKE_WIDTH}
          strokeLinecap="round"
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={offset}
          className="stroke-foreground transition-[stroke-dashoffset] duration-1000 ease-linear motion-reduce:transition-none"
        />
      </svg>
      <span aria-hidden="true" className="absolute text-[10px] font-medium tabular-nums">{secondsLeft}</span>
    </span>
  );
}
