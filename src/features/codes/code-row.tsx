import type { CodeView } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useCopyCode, type CopyState } from "./use-copy-code";

const STATUS_TEXT: Record<CopyState, string> = {
  idle: "",
  copied: "Copied",
  failed: "Copy failed",
};

interface CodeRowProps {
  view: CodeView;
  accountIndex: number;
}

export function CodeRow({ view, accountIndex }: CodeRowProps) {
  const { copyState, copy } = useCopyCode(accountIndex);
  const hasStatus = copyState !== "idle";
  return (
    <li className="border-b last:border-b-0">
      <button
        type="button"
        onClick={copy}
        aria-label={`Copy code for ${view.label}`}
        className="flex w-full items-baseline justify-between gap-4 rounded-md py-3 text-left outline-none hover:bg-muted/50 focus-visible:ring-3 focus-visible:ring-ring/50"
      >
        <span className="truncate text-sm">{view.label}</span>
        <span className="flex items-baseline gap-2 tabular-nums">
          <span className="text-2xl font-semibold tracking-wider">{view.code}</span>
          <span className={cn("text-xs text-muted-foreground", copyState === "failed" && "text-destructive")}>
            {hasStatus ? STATUS_TEXT[copyState] : `${view.seconds_left}s`}
          </span>
        </span>
      </button>
      <span role="status" className="sr-only">{STATUS_TEXT[copyState]}</span>
    </li>
  );
}
