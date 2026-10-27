import { Eye, EyeOff } from "lucide-react";
import { IconButton } from "@/components/icon-button";

interface RevealButtonProps {
  accountLabel: string;
  isRevealed: boolean;
  onToggle: () => void;
}

export function RevealButton({ accountLabel, isRevealed, onToggle }: RevealButtonProps) {
  return (
    <IconButton
      label={`${isRevealed ? "Hide" : "Reveal"} code for ${accountLabel}`}
      variant="ghost"
      aria-pressed={isRevealed}
      onClick={onToggle}
    >
      {isRevealed ? <EyeOff /> : <Eye />}
    </IconButton>
  );
}
