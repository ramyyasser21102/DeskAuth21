import { labelLines } from "./label-lines";

/** The service on top and the account name beneath, each cut off on its own line. */
/** `wrap` lets long names run onto a second line instead of being cut off. */
export function AccountLabel({ issuer, name, wrap }: { issuer: string; name: string; wrap?: boolean }) {
  const lineClass = wrap ? "break-words" : "truncate";
  const { primary, secondary } = labelLines(issuer, name);
  return (
    <span className="flex min-w-0 flex-col">
      <span className={`${lineClass} text-sm font-medium`} title={primary}>{primary}</span>
      {secondary && <span className={`${lineClass} text-xs text-muted-foreground`} title={secondary}>{secondary}</span>}
    </span>
  );
}
