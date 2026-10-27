import { Monitor, Moon, Sun, type LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTheme } from "./theme-provider";
import type { ThemePreference } from "./theme-preference";

const THEME_OPTIONS: readonly { value: ThemePreference; label: string; Icon: LucideIcon }[] = [
  { value: "system", label: "System theme", Icon: Monitor },
  { value: "light", label: "Light theme", Icon: Sun },
  { value: "dark", label: "Dark theme", Icon: Moon },
];

export function ThemeToggle() {
  const { preference, setPreference } = useTheme();
  return (
    <div role="group" aria-label="Colour theme" className="flex gap-1">
      {THEME_OPTIONS.map(({ value, label, Icon }) => (
        <Button
          key={value}
          type="button"
          size="icon-sm"
          variant={preference === value ? "secondary" : "ghost"}
          aria-label={label}
          aria-pressed={preference === value}
          title={label}
          onClick={() => setPreference(value)}
        >
          <Icon />
        </Button>
      ))}
    </div>
  );
}
