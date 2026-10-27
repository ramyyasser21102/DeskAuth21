import { useState } from "react";
import { Button } from "@/components/ui/button";
import { UseExistingVaultButton } from "@/features/sync/use-existing-vault-button";
import { createVault, unlockVault } from "@/lib/api";
import { PassphraseForm } from "./passphrase-form";
import { PasswordField } from "./password-field";
import { useUnlockNeedsHardwareKey } from "./use-unlock-needs-hardware-key";
import { RecoveryUnlockForm } from "./recovery-unlock-form";

const KEY_HINT = "This vault also needs your hardware key. Plug it in, enter your passphrase, then touch the key when it blinks.";
const TOUCH_MESSAGE = "Touch your key when it blinks…";

const CREATE_VAULT_HINT =
  "Choose a passphrase of at least 12 characters, or generate one. It cannot be recovered if you forget it.";

export function CreateVaultScreen({ onCreated }: { onCreated: () => void }) {
  const handleSubmit = async (passphrase: string) => {
    await createVault(passphrase);
    onCreated();
  };
  return (
    <div className="flex flex-col gap-4">
      <PassphraseForm
        title="Create your vault"
        hint={CREATE_VAULT_HINT}
        submitLabel="Create vault"
        requiresConfirmation
        onSubmit={handleSubmit}
      />
      <UseExistingVaultButton onChosen={onCreated} />
    </div>
  );
}

export function UnlockScreen({ onUnlocked }: { onUnlocked: () => void }) {
  const needsKey = useUnlockNeedsHardwareKey();
  const [keyPin, setKeyPin] = useState("");
  const handleSubmit = async (passphrase: string) => {
    await unlockVault(passphrase, keyPin === "" ? undefined : keyPin);
    onUnlocked();
  };
  const [isUsingRecoveryKey, setIsUsingRecoveryKey] = useState(false);
  if (isUsingRecoveryKey) return <RecoveryUnlockForm onUnlocked={onUnlocked} onBack={() => setIsUsingRecoveryKey(false)} />;
  return (
    <div className="flex flex-col gap-4">
      <PassphraseForm title="Unlock DA21" submitLabel="Unlock" requiresConfirmation={false} onSubmit={handleSubmit}
        hint={needsKey ? KEY_HINT : undefined} busyMessage={needsKey ? TOUCH_MESSAGE : undefined}>
        {needsKey && <PasswordField id="key-pin" label="Key PIN (only if your key has one)" value={keyPin} onChange={setKeyPin} />}
      </PassphraseForm>
      <Button variant="ghost" onClick={() => setIsUsingRecoveryKey(true)}>Forgot your passphrase? Use your recovery key</Button>
    </div>
  );
}
