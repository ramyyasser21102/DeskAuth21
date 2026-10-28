import { useState } from "react";
import { Button } from "@/components/ui/button";
import { restoreVaultBackup } from "@/lib/api";
import { RestoreConfirmation } from "./restore-confirmation";

interface RestoreBackupButtonProps {
  /** True when a vault is already here, so restoring would replace it and must be confirmed. */
  replacesVault: boolean;
  onRestored: () => void;
}

/** For the first screens: bring a vault back from a backup file before anything is unlocked. */
export function RestoreBackupButton({ replacesVault, onRestored }: RestoreBackupButtonProps) {
  const [isConfirming, setIsConfirming] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const restore = async () => {
    setIsConfirming(false);
    try {
      if (await restoreVaultBackup()) onRestored();
    } catch (error) {
      setErrorMessage(String(error));
    }
  };

  if (isConfirming) return <RestoreConfirmation onConfirm={() => void restore()} onCancel={() => setIsConfirming(false)} />;
  return (
    <div className="flex flex-col items-center gap-1">
      <Button type="button" variant="ghost" onClick={() => (replacesVault ? setIsConfirming(true) : void restore())}>
        Restore from a backup file
      </Button>
      {errorMessage && <p role="alert" className="text-xs text-destructive">{errorMessage}</p>}
    </div>
  );
}
