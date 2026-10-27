import { Monitor, Moon, Sun, type LucideIcon } from "lucide-react";
import { IconButton } from "@/components/icon-button";
import { useTheme } from "./theme-provider";
import type { ThemePreference } from "./theme-preference";

const THEME_OPTIONS: readonly { value: ThemePreference; label: string; Icon: LucideIcon }[] = [
  { value: "system", label: "Use system theme", Icon: Monitor },
  { value: "light", label: "Use light theme", Icon: Sun },
  { value: "dark", label: "Use dark theme", Icon: Moon },
];

export function ThemeToggle() {
  const { preference, setPreference } = useTheme();
  return (
    <div role="group" aria-label="Colour theme" className="flex gap-1">
      {THEME_OPTIONS.map(({ value, label, Icon }) => (
        <IconButton
          key={value}
          label={label}
          variant={preference === value ? "secondary" : "ghost"}
          aria-pressed={preference === value}
          onClick={() => setPreference(value)}
        >
          <Icon />
        </IconButton>
      ))}
    </div>
  );
}
