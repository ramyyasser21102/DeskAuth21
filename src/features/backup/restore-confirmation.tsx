import { Button } from "@/components/ui/button";

interface RestoreConfirmationProps {
  onConfirm: () => void;
  onCancel: () => void;
}

const EXPLANATION = "Restoring replaces the vault on this computer with the backup you pick, then locks. The vault it replaces is kept next to it. You will unlock with the backup's passphrase.";

/** Replacing the vault is hard to notice going wrong, so it asks first and starts on Cancel. */
export function RestoreConfirmation({ onConfirm, onCancel }: RestoreConfirmationProps) {
  return (
    <div className="flex flex-col gap-3" onKeyDown={(event) => event.key === "Escape" && onCancel()}>
      <p className="text-sm">{EXPLANATION}</p>
      <div className="flex gap-2">
        <Button type="button" variant="outline" autoFocus onClick={onCancel}>Cancel</Button>
        <Button type="button" variant="destructive" onClick={onConfirm}>Choose a backup…</Button>
      </div>
    </div>
  );
}
