import { useEffect, useRef } from "react";

/**
 * Hands keyboard focus back to the control that opened something once it closes.
 * Attach the returned ref to that control.
 */
export function useFocusReturn<Element extends HTMLElement>(isOpen: boolean) {
  const triggerRef = useRef<Element>(null);
  const wasOpen = useRef(false);
  useEffect(() => {
    if (wasOpen.current && !isOpen) triggerRef.current?.focus();
    wasOpen.current = isOpen;
  }, [isOpen]);
  return triggerRef;
}
