import { Button } from "@/components/ui/button";
import { CardFooter } from "@/components/ui/card";
import { lockVault } from "@/lib/api";

interface CodesFooterProps {
  isImportOpen: boolean;
  onToggleImport: () => void;
  onLocked: () => void;
}

export function CodesFooter({ isImportOpen, onToggleImport, onLocked }: CodesFooterProps) {
  const handleLock = async () => {
    await lockVault();
    onLocked();
  };
  return (
    <CardFooter className="gap-2">
      <Button variant="outline" onClick={onToggleImport}>
        {isImportOpen ? "Hide import" : "Add accounts"}
      </Button>
      <Button variant="outline" onClick={handleLock}>Lock</Button>
    </CardFooter>
  );
}
