import { useState } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { CURRENT_FORMAT_VERSION, useVaultFormat } from "./use-vault-format";

const EXPLANATION =
  "The new format lets you add a recovery key and, later, a hardware key. A copy of the current file is kept beside it. Computers that share this vault must update DA21 first, or they will not be able to open it.";

/** Offered on every unlock while the vault is in an older format; "Not now" only hides it until next time. */
export function UpgradePrompt() {
  const { version, isUpgrading, errorMessage, upgrade } = useVaultFormat();
  const [isDismissed, setIsDismissed] = useState(false);
  const [didUpgrade, setDidUpgrade] = useState(false);

  const handleUpgrade = async () => {
    await upgrade();
    setDidUpgrade(true);
  };

  if (didUpgrade && version === CURRENT_FORMAT_VERSION) {
    return <p role="status" className="mb-4 text-sm text-muted-foreground">Vault upgraded. The old file was kept beside it as vault.da21.v1.bak.</p>;
  }
  if (version === null || version >= CURRENT_FORMAT_VERSION || isDismissed) return null;

  return (
    <Alert className="mb-4">
      <AlertTitle>Upgrade your vault's security format</AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-3">
        {EXPLANATION}
        {errorMessage && <span role="alert" className="text-destructive">{errorMessage}</span>}
        <div className="flex gap-2">
          <Button size="sm" disabled={isUpgrading} onClick={() => void handleUpgrade()}>Upgrade vault</Button>
          <Button size="sm" variant="outline" disabled={isUpgrading} onClick={() => setIsDismissed(true)}>Not now</Button>
        </div>
      </AlertDescription>
    </Alert>
  );
}
