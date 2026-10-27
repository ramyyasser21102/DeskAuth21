import { useEffect } from "react";
import { recordActivity } from "@/lib/api";
import { throttle } from "@/lib/throttle";

const REPORT_INTERVAL_MS = 5000;
const ACTIVITY_EVENTS = ["pointerdown", "pointermove", "keydown", "wheel"] as const;

/**
 * Tells the backend the user is present, so the auto-lock timer restarts. Input
 * is throttled; reporting is best-effort and a failure is not worth surfacing.
 */
export function useActivityReporter(): void {
  useEffect(() => {
    const report = throttle(() => void recordActivity().catch(() => {}), REPORT_INTERVAL_MS);
    ACTIVITY_EVENTS.forEach((name) => document.addEventListener(name, report, { passive: true }));
    return () => ACTIVITY_EVENTS.forEach((name) => document.removeEventListener(name, report));
  }, []);
}
