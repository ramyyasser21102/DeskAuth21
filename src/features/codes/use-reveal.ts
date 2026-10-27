import { useCallback, useEffect, useState } from "react";
import { REVEAL_SECONDS } from "@/features/privacy/privacy-state";

const MILLISECONDS_PER_SECOND = 1000;

/**
 * Whether one code is currently shown as digits. A reveal ends by itself after
 * `REVEAL_SECONDS`, and at once when the window loses focus or is hidden, so
 * digits are never left on screen when the person looks away.
 */
export function useReveal() {
  const [isRevealed, setIsRevealed] = useState(false);
  const hide = useCallback(() => setIsRevealed(false), []);
  const toggle = useCallback(() => setIsRevealed((current) => !current), []);

  useEffect(() => {
    if (!isRevealed) return;
    const timerId = window.setTimeout(hide, REVEAL_SECONDS * MILLISECONDS_PER_SECOND);
    window.addEventListener("blur", hide);
    document.addEventListener("visibilitychange", hide);
    return () => {
      window.clearTimeout(timerId);
      window.removeEventListener("blur", hide);
      document.removeEventListener("visibilitychange", hide);
    };
  }, [isRevealed, hide]);

  return { isRevealed, toggle };
}
