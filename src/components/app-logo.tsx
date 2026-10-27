import { cn } from "@/lib/utils";

/**
 * A black tile holding a timer ring with a gap, the way a code's countdown looks.
 * `currentColor` for the tile and the page background for the ring keep it black
 * and white in both themes.
 */
export function AppLogo({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 64 64" role="img" aria-label="DeskAuth21 logo" className={cn("size-10", className)}>
      <rect width="64" height="64" rx="14" className="fill-foreground" />
      <circle cx="32" cy="32" r="16" fill="none" strokeWidth="5" strokeLinecap="round" strokeDasharray="76 25" transform="rotate(-60 32 32)" className="stroke-background" />
      <circle cx="32" cy="32" r="4" className="fill-background" />
    </svg>
  );
}
