import { Button } from "@/components/ui/button";
import { AUTO_LOCK_OPTIONS, describeAutoLock } from "./auto-lock-options";
import { useAutoLockSetting } from "./use-auto-lock-setting";

export function AutoLockSetting() {
  const { minutes, errorMessage, choose } = useAutoLockSetting();
  if (minutes === undefined) return null;
  return (
    <div className="flex flex-col gap-2">
      <span id="auto-lock-label" className="text-sm font-medium">Auto-lock</span>
      <div role="group" aria-labelledby="auto-lock-label" className="flex flex-wrap gap-1">
        {AUTO_LOCK_OPTIONS.map((option) => (
          <Button
            key={option.label}
            type="button"
            size="sm"
            variant={option.minutes === minutes ? "secondary" : "ghost"}
            aria-pressed={option.minutes === minutes}
            onClick={() => choose(option.minutes)}
          >
            {option.label}
          </Button>
        ))}
      </div>
      <p className="text-xs text-muted-foreground">{errorMessage ?? describeAutoLock(minutes)}</p>
    </div>
  );
}
