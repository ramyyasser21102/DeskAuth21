import { useEffect, useState } from "react";
import { copyCode } from "@/lib/api";

const FEEDBACK_DURATION_MS = 2000;

export type CopyState = "idle" | "copied" | "failed";

/** Copies one account's code through the backend (which also clears it later). */
export function useCopyCode(accountId: string, onCopied?: () => void) {
  const [copyState, setCopyState] = useState<CopyState>("idle");

  useEffect(() => {
    if (copyState === "idle") return;
    const timerId = window.setTimeout(() => setCopyState("idle"), FEEDBACK_DURATION_MS);
    return () => window.clearTimeout(timerId);
  }, [copyState]);

  const copy = async () => {
    try {
      await copyCode(accountId);
      setCopyState("copied");
      onCopied?.();
    } catch {
      setCopyState("failed");
    }
  };

  return { copyState, copy };
}
