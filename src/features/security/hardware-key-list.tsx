import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import type { HardwareKeyEntry } from "@/lib/api";
import { RemovalConfirmation } from "./hardware-key-removal";
import { useRemoveHardwareKey } from "./use-remove-hardware-key";

interface HardwareKeyListProps {
  keys: readonly HardwareKeyEntry[];
  onChanged: () => void;
}

/** Removing a key asks first and names it; the last key cannot be removed. */
export function HardwareKeyList({ keys, onChanged }: HardwareKeyListProps) {
  const removal = useRemoveHardwareKey(onChanged);
  const canRemove = keys.length > 1;
  return (
    <div className="flex flex-col gap-2">
      <ul className="flex flex-col gap-2">
        {keys.map((key) => (
          <li key={key.id} className="flex flex-col gap-2 rounded-md border p-3">
            <div className="flex items-center justify-between gap-2">
              <span className="text-sm font-medium">{key.label}</span>
              {canRemove && removal.pendingId !== key.id && (
                <Button type="button" size="sm" variant="ghost" onClick={() => removal.askToRemove(key.id)}>{`Remove ${key.label}`}</Button>
              )}
            </div>
            {removal.pendingId === key.id && (
              <RemovalConfirmation label={key.label} onKeep={() => removal.askToRemove(null)} onRemove={() => void removal.remove(key.id)} />
            )}
          </li>
        ))}
      </ul>
      {!canRemove && <p className="text-xs text-muted-foreground">A vault needs at least one key. Add another before removing this one, or stop requiring a key.</p>}
      {removal.errorMessage && <Alert variant="destructive"><AlertDescription>{removal.errorMessage}</AlertDescription></Alert>}
    </div>
  );
}
