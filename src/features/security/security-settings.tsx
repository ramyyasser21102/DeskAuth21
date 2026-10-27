import { useState } from "react";
import { Button } from "@/components/ui/button";
import { ChangePassphraseForm } from "./change-passphrase-form";
import { RecoveryKeySetting } from "./recovery-key-setting";
import { CURRENT_FORMAT_VERSION, useVaultFormat } from "./use-vault-format";

const NEEDS_UPGRADE = "Changing the passphrase and creating a recovery key need the newer vault format. Use the notice at the top of the window to upgrade first.";

/** Everything that protects the vault file itself, kept apart from the display privacy choices. */
export function SecuritySettings() {
  const { version } = useVaultFormat();
  const [isChanging, setIsChanging] = useState(false);
  if (version === null) return null;
  if (version < CURRENT_FORMAT_VERSION) {
    return <div className="flex flex-col gap-2"><span className="text-sm font-medium">Security</span><p className="text-xs text-muted-foreground">{NEEDS_UPGRADE}</p></div>;
  }
  return (
    <div className="flex flex-col gap-4">
      <span className="text-sm font-medium">Security</span>
      {isChanging ? (
        <ChangePassphraseForm onClose={() => setIsChanging(false)} />
      ) : (
        <Button type="button" variant="outline" size="sm" className="self-start" onClick={() => setIsChanging(true)}>Change passphrase</Button>
      )}
      <RecoveryKeySetting />
    </div>
  );
}
