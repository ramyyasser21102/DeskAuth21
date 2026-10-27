import { Search, X } from "lucide-react";
import { IconButton } from "@/components/icon-button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

const SEARCH_ID = "code-search";

interface CodeSearchProps {
  query: string;
  onChange: (query: string) => void;
}

/** Filters as you type; Escape clears it. The label is visible so the box is never anonymous. */
export function CodeSearch({ query, onChange }: CodeSearchProps) {
  return (
    <div className="flex flex-col gap-1">
      <Label htmlFor={SEARCH_ID}>Search accounts</Label>
      <div className="relative">
        <Search className="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
        <Input
          id={SEARCH_ID}
          type="search"
          autoComplete="off"
          spellCheck={false}
          className="pl-8"
          value={query}
          placeholder="Name, account or app"
          onChange={(event) => onChange(event.target.value)}
          onKeyDown={(event) => event.key === "Escape" && onChange("")}
        />
        {query !== "" && (
          <IconButton label="Clear search" variant="ghost" className="absolute right-0 top-1/2 size-9 -translate-y-1/2" onClick={() => onChange("")}>
            <X />
          </IconButton>
        )}
      </div>
    </div>
  );
}
