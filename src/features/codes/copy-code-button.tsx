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

/** The code and its countdown; clicking copies it (and the backend clears it later). */
export function CopyCodeButton({ view, isCodeHidden }: { view: CodeView; isCodeHidden: boolean }) {
  const { copyState, copy } = useCopyCode(view.id);
  const hasStatus = copyState !== "idle";
  return (
    <>
      <button
        type="button"
        onClick={copy}
        aria-label={`Copy code for ${view.label}`}
        className="flex min-w-0 flex-1 items-center justify-between gap-3 rounded-md px-2 py-3 text-left outline-none transition-colors hover:bg-muted/50 focus-visible:ring-3 focus-visible:ring-ring/50 active:bg-muted"
      >
        <AccountLabel issuer={view.issuer} name={view.name} />
        <span className="flex items-center gap-3 tabular-nums">
          {/* A new key per code replays the fade, so a changed code is noticed. */}
          <span key={view.code} className="animate-in fade-in text-xl font-semibold tracking-wide duration-300 sm:text-2xl sm:tracking-wider motion-reduce:animate-none">
            {isCodeHidden ? maskCode(view.code) : view.code}
          </span>
          {hasStatus ? (
            <span className={cn("w-12 text-xs text-muted-foreground", copyState === "failed" && "text-destructive")}>{STATUS_TEXT[copyState]}</span>
          ) : (
            <CountdownRing secondsLeft={view.seconds_left} periodSeconds={view.period_seconds} />
          )}
        </span>
      </button>
      <span role="status" className="sr-only">{STATUS_TEXT[copyState]}</span>
    </>
  );
}
