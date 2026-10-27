import { Search } from "lucide-react";
import type { KeyboardEvent, Ref } from "react";
import { Input } from "@/components/ui/input";

interface PanelSearchProps {
  query: string;
  onChange: (query: string) => void;
  onEnter: () => void;
  inputRef: Ref<HTMLInputElement>;
}

/** The box the cursor starts in: type a few letters, press Enter to copy the first match. */
export function PanelSearch({ query, onChange, onEnter, inputRef }: PanelSearchProps) {
  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") onEnter();
  };
  return (
    <div className="relative p-2">
      <Search className="pointer-events-none absolute left-4 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
      <Input
        ref={inputRef}
        type="search"
        aria-label="Search accounts"
        placeholder="Search accounts"
        autoComplete="off"
        spellCheck={false}
        className="pl-8"
        value={query}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={handleKeyDown}
      />
    </div>
  );
}
