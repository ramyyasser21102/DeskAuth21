import { Button } from "@/components/ui/button";
import { CLIPBOARD_CLEAR_OPTIONS, describeClipboardClearing } from "./clipboard-options";

interface ClipboardClearSettingProps {
  seconds: number;
  onChoose: (seconds: number) => void;
}

export function ClipboardClearSetting({ seconds, onChoose }: ClipboardClearSettingProps) {
  return (
    <div className="flex flex-col gap-2">
      <span id="clipboard-clear-label" className="text-sm font-medium">Clear copied codes after</span>
      <div role="group" aria-labelledby="clipboard-clear-label" className="flex flex-wrap gap-1">
        {CLIPBOARD_CLEAR_OPTIONS.map((option) => (
          <Button key={option} type="button" size="sm" variant={option === seconds ? "secondary" : "ghost"} aria-pressed={option === seconds} onClick={() => onChoose(option)}>
            {option} s
          </Button>
        ))}
      </div>
      <p className="text-xs text-muted-foreground">{describeClipboardClearing(seconds)}</p>
    </div>
  );
}
