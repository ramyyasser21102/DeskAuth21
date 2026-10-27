import { useState } from "react";
import { Button } from "@/components/ui/button";
import { chooseVaultFolder } from "@/lib/api";

/** For a new device: point the app at a vault that already exists in a synced folder. */
export function UseExistingVaultButton({ onChosen }: { onChosen: () => void }) {
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handleClick = async () => {
    try {
      if ((await chooseVaultFolder()) !== null) onChosen();
    } catch (error) {
      setErrorMessage(String(error));
    }
  };

  return (
    <div className="flex flex-col items-center gap-1">
      <Button type="button" variant="ghost" onClick={handleClick}>
        Open a vault from a synced folder
      </Button>
      {errorMessage && <p className="text-xs text-destructive">{errorMessage}</p>}
    </div>
  );
}
