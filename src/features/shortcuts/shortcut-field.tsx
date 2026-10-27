import { useState, type KeyboardEvent } from "react";
import { Button } from "@/components/ui/button";
import { describeShortcut, shortcutFromKeyPress } from "./shortcut-keys";

interface ShortcutFieldProps {
  id: string;
  label: string;
  description: string;
  shortcut: string | null;
  onChoose: (shortcut: string | null) => void;
}

/** After a click, the next key combination is reported once; Escape gives up. */
function useShortcutListener(onChoose: (shortcut: string) => void) {
  const [isListening, setIsListening] = useState(false);
  const handleKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (!isListening) return;
    event.preventDefault();
    if (event.key === "Escape") return setIsListening(false);
    const pressed = shortcutFromKeyPress(event);
    if (pressed === null) return;
    setIsListening(false);
    onChoose(pressed);
  };
  return { isListening, startListening: () => setIsListening(true), stopListening: () => setIsListening(false), handleKeyDown };
}

export function ShortcutField({ id, label, description, shortcut, onChoose }: ShortcutFieldProps) {
  const { isListening, startListening, stopListening, handleKeyDown } = useShortcutListener(onChoose);

  return (
    <div className="flex flex-col gap-2">
      <span id={`${id}-label`} className="text-sm font-medium">{label}</span>
      <div className="flex items-center gap-2">
        <Button
          id={id}
          type="button"
          size="sm"
          variant="outline"
          aria-labelledby={`${id}-label ${id}`}
          onClick={startListening}
          onBlur={stopListening}
          onKeyDown={handleKeyDown}
        >
          {isListening ? "Press the keys, Esc to cancel" : shortcut ? describeShortcut(shortcut) : "Not set"}
        </Button>
        {shortcut && <Button type="button" size="sm" variant="ghost" onClick={() => onChoose(null)}>Clear</Button>}
      </div>
      <p className="text-xs text-muted-foreground">{description}</p>
    </div>
  );
}
