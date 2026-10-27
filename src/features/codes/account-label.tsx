import { labelLines } from "./label-lines";

/** The service on top and the account name beneath, each cut off on its own line. */
export function AccountLabel({ issuer, name }: { issuer: string; name: string }) {
  const { primary, secondary } = labelLines(issuer, name);
  return (
    <span className="flex min-w-0 flex-col">
      <span className="truncate text-sm font-medium" title={primary}>{primary}</span>
      {secondary && <span className="truncate text-xs text-muted-foreground" title={secondary}>{secondary}</span>}
    </span>
  );
}
