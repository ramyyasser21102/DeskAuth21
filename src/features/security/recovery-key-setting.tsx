import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { hasRecoveryKey } from "@/lib/api";
import { RecoveryKeyWizard } from "./recovery-key-wizard";

const WHY = "A recovery key is 24 words that open your vault if you forget your passphrase. Without one, a forgotten passphrase cannot be recovered.";

export function RecoveryKeySetting() {
  const [isSet, setIsSet] = useState<boolean | null>(null);
  const [isWizardOpen, setIsWizardOpen] = useState(false);

  const refresh = useCallback(() => {
    hasRecoveryKey().then(setIsSet).catch(() => setIsSet(null));
  }, []);
  useEffect(refresh, [refresh]);

  const close = () => {
    setIsWizardOpen(false);
    refresh();
  };

  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium">Recovery key: {isSet === null ? "checking" : isSet ? "set" : "not set"}</span>
      {isWizardOpen ? (
        <RecoveryKeyWizard onClose={close} />
      ) : (
        <>
          <p className="text-xs text-muted-foreground">{WHY}</p>
          <Button type="button" variant="outline" size="sm" className="self-start" onClick={() => setIsWizardOpen(true)}>
            {isSet ? "Replace recovery key" : "Create recovery key"}
          </Button>
        </>
      )}
    </div>
  );
}
