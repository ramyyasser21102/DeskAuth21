import { Check } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { SWAP_MOTION, TAP_MOTION, PAGE_MOTION } from "@/lib/motion-tokens";
import type { CodeView } from "@/lib/api";
import { cn } from "@/lib/utils";
import { maskCode } from "@/features/privacy/privacy-state";
import { AccountLabel } from "./account-label";
import { CountdownRing } from "./countdown-ring";
import { useCopyCode, type CopyState } from "./use-copy-code";

const STATUS_TEXT: Record<CopyState, string> = {
  idle: "",
  copied: "Copied",
  failed: "Copy failed",
};

/** Takes the ring's place for a moment, with a check when the copy worked. */
function CopyStatus({ copyState }: { copyState: CopyState }) {
  return (
    <motion.span aria-hidden="true" className={cn("flex w-12 items-center gap-1 text-xs text-muted-foreground", copyState === "failed" && "text-destructive")} {...SWAP_MOTION}>
      {copyState === "copied" && <Check className="size-3.5" />}
      {STATUS_TEXT[copyState]}
    </motion.span>
  );
}

/** The code and its countdown; clicking copies it (and the backend clears it later). */
export function CopyCodeButton({ view, isCodeHidden }: { view: CodeView; isCodeHidden: boolean }) {
  const { copyState, copy } = useCopyCode(view.id);
  const hasStatus = copyState !== "idle";
  return (
    <>
      <motion.button
        type="button"
        whileTap={TAP_MOTION}
        onClick={copy}
        aria-label={`Copy code for ${view.label}`}
        className="flex min-w-0 flex-1 items-center justify-between gap-3 rounded-md px-2 py-3 text-left outline-none transition-colors hover:bg-muted/50 focus-visible:ring-3 focus-visible:ring-ring/50 active:bg-muted"
      >
        <AccountLabel issuer={view.issuer} name={view.name} />
        <span className="flex items-center gap-3 tabular-nums">
          {/* A new key per code replays the fade, so a changed code is noticed. */}
          <motion.span key={view.code} className="text-xl font-semibold tracking-wide sm:text-2xl sm:tracking-wider" {...PAGE_MOTION} initial={{ opacity: 0 }}>
            {isCodeHidden ? maskCode(view.code) : view.code}
          </motion.span>
          <AnimatePresence mode="wait" initial={false}>
            {hasStatus ? <CopyStatus key={copyState} copyState={copyState} /> : <motion.span key="ring" {...SWAP_MOTION}><CountdownRing secondsLeft={view.seconds_left} periodSeconds={view.period_seconds} /></motion.span>}
          </AnimatePresence>
        </span>
      </motion.button>
      <span role="status" className="sr-only">{STATUS_TEXT[copyState]}</span>
    </>
  );
}
