import { createVault, unlockVault } from "@/lib/api";
import { PassphraseForm } from "./passphrase-form";

const CREATE_VAULT_HINT =
  "Choose a passphrase of at least 8 characters. It cannot be recovered if you forget it.";

export function CreateVaultScreen({ onCreated }: { onCreated: () => void }) {
  const handleSubmit = async (passphrase: string) => {
    await createVault(passphrase);
    onCreated();
  };
  return (
    <PassphraseForm
      title="Create your vault"
      hint={CREATE_VAULT_HINT}
      submitLabel="Create vault"
      requiresConfirmation
      onSubmit={handleSubmit}
    />
  );
}

export function UnlockScreen({ onUnlocked }: { onUnlocked: () => void }) {
  const handleSubmit = async (passphrase: string) => {
    await unlockVault(passphrase);
    onUnlocked();
  };
  return <PassphraseForm title="Unlock DeskAuth21" submitLabel="Unlock" requiresConfirmation={false} onSubmit={handleSubmit} />;
}
