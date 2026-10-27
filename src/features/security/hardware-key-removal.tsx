import { Button } from "@/components/ui/button";

interface RemovalConfirmationProps {
  label: string;
  onKeep: () => void;
  onRemove: () => void;
}

/** Names the key, starts on the safe choice, and Escape picks it. */
export function RemovalConfirmation({ label, onKeep, onRemove }: RemovalConfirmationProps) {
  return (
    <div role="group" aria-label={`Remove ${label}?`} onKeyDown={(event) => event.key === "Escape" && onKeep()} className="flex flex-col gap-2">
      <p className="text-sm">{`Remove ${label}? It will no longer open this vault.`}</p>
      <div className="flex gap-2">
        <Button type="button" size="sm" variant="outline" autoFocus onClick={onKeep}>Keep key</Button>
        <Button type="button" size="sm" variant="destructive" onClick={onRemove}>{`Remove ${label}`}</Button>
      </div>
    </div>
  );
}
