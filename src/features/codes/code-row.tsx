import type { CodeView } from "@/lib/api";

export function CodeRow({ view }: { view: CodeView }) {
  return (
    <li className="flex items-baseline justify-between gap-4 border-b py-3 last:border-b-0">
      <span className="truncate text-sm">{view.label}</span>
      <span className="flex items-baseline gap-2 tabular-nums">
        <span className="text-2xl font-semibold tracking-wider">{view.code}</span>
        <span className="text-xs text-muted-foreground">{view.seconds_left}s</span>
      </span>
    </li>
  );
}
