import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";

const DELETE_WARNING =
  "This also removes it from your other synced devices. Make sure you can still sign in to that service another way, such as a backup code, or you could lose access.";

interface DeleteConfirmationProps {
  accountLabel: string;
  isBusy: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function DeleteConfirmation({ accountLabel, isBusy, onConfirm, onCancel }: DeleteConfirmationProps) {
  return (
    <div className="flex flex-col gap-3 py-3" onKeyDown={(event) => event.key === "Escape" && onCancel()}>
      <p className="text-sm font-medium">Delete {accountLabel}?</p>
      <Alert variant="destructive">
        <AlertDescription>{DELETE_WARNING}</AlertDescription>
      </Alert>
      <div className="flex gap-2">
        <Button type="button" variant="destructive" disabled={isBusy} onClick={onConfirm}>Delete account</Button>
        <Button type="button" variant="outline" autoFocus disabled={isBusy} onClick={onCancel}>Keep account</Button>
      </div>
    </div>
  );
}
