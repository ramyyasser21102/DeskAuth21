/**
 * Wraps `action` so it runs at most once per `intervalMs`. The clock is passed in
 * so the behaviour can be tested without waiting.
 */
export function throttle(action: () => void, intervalMs: number, now: () => number = Date.now): () => void {
  let lastRunAt = Number.NEGATIVE_INFINITY;
  return () => {
    const currentTime = now();
    if (currentTime - lastRunAt < intervalMs) return;
    lastRunAt = currentTime;
    action();
  };
}
