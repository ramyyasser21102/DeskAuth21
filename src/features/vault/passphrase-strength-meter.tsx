import { cn } from "@/lib/utils";
import { usePassphraseStrength } from "./use-passphrase-strength";

const STRENGTH_LABELS = ["Very weak", "Weak", "Fair", "Good", "Strong"] as const;
const HIGHEST_SCORE = STRENGTH_LABELS.length - 1;

/** An advisory estimate, not a gate: the length rule is what is enforced. */
export function PassphraseStrengthMeter({ passphrase }: { passphrase: string }) {
  const strength = usePassphraseStrength(passphrase);
  if (!strength) return null;
  const label = STRENGTH_LABELS[Math.min(strength.score, HIGHEST_SCORE)];
  return (
    <div className="flex flex-col gap-1">
      <div role="meter" aria-label="Passphrase strength" aria-valuemin={0} aria-valuemax={HIGHEST_SCORE} aria-valuenow={strength.score} aria-valuetext={label} className="flex gap-1">
        {STRENGTH_LABELS.map((segment, index) => (
          <span key={segment} className={cn("h-1.5 flex-1 rounded-full transition-colors duration-200", index <= strength.score ? "bg-foreground" : "bg-border")} />
        ))}
      </div>
      <p className="text-xs text-muted-foreground">{strength.feedback ? `${label}. ${strength.feedback}` : label}</p>
    </div>
  );
}
