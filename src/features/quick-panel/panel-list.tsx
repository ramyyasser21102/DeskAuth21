import type { VendorGroup } from "@/features/vendors/search-and-group";
import { PanelRow } from "./panel-row";

interface PanelListProps {
  groups: readonly VendorGroup[];
  hasAccounts: boolean;
  query: string;
  onCopied: () => void;
}

/** Every account that matches, under a heading for its app, in a list that scrolls. */
export function PanelList({ groups, hasAccounts, query, onCopied }: PanelListProps) {
  if (groups.length === 0) {
    const message = hasAccounts ? `No accounts match "${query.trim()}".` : "No accounts yet.";
    return <p className="flex-1 px-3 py-6 text-center text-sm text-muted-foreground">{message}</p>;
  }
  return (
    <div className="flex-1 overflow-y-auto px-1">
      {groups.map((group) => (
        <section key={group.vendor} aria-labelledby={`panel-group-${group.vendor}`}>
          <h2 id={`panel-group-${group.vendor}`} className="px-2 pt-3 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            {group.name} ({group.views.length})
          </h2>
          <ul>
            {group.views.map((view) => <PanelRow key={view.id} view={view} onCopied={onCopied} />)}
          </ul>
        </section>
      ))}
    </div>
  );
}
