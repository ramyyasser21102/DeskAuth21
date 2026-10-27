import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";

interface CaptureOffConfirmationProps {
  onTurnOff: () => void;
  onKeep: () => void;
}

/** The safe choice has focus and Escape picks it, so a stray key never lowers protection. */
export function CaptureOffConfirmation({ onTurnOff, onKeep }: CaptureOffConfirmationProps) {
  return (
    <Alert variant="destructive" onKeyDown={(event) => event.key === "Escape" && onKeep()}>
      <AlertTitle>Turn off screen capture protection?</AlertTitle>
      <AlertDescription className="flex flex-col gap-3">
        Screenshots, recordings and screen sharing will be able to capture your codes.
        <div className="flex gap-2">
          <Button size="sm" variant="outline" autoFocus onClick={onKeep}>Keep protection</Button>
          <Button size="sm" variant="destructive" onClick={onTurnOff}>Turn off protection</Button>
        </div>
      </AlertDescription>
    </Alert>
  );
}
