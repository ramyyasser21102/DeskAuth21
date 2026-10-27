import { createVault, unlockVault } from "../../lib/api";
import { renderPassphraseForm } from "./passphrase-form";

const CREATE_VAULT_HINT =
  "Choose a passphrase of at least 8 characters. It cannot be recovered if you forget it.";

export function renderCreateVaultScreen(container: HTMLElement, onCreated: () => void): void {
  renderPassphraseForm(container, {
    title: "Create your vault",
    hint: CREATE_VAULT_HINT,
    submitLabel: "Create vault",
    requiresConfirmation: true,
    onSubmit: async (passphrase) => {
      await createVault(passphrase);
      onCreated();
    },
  });
}

export function renderUnlockScreen(container: HTMLElement, onUnlocked: () => void): void {
  renderPassphraseForm(container, {
    title: "Unlock DeskAuth21",
    submitLabel: "Unlock",
    requiresConfirmation: false,
    onSubmit: async (passphrase) => {
      await unlockVault(passphrase);
      onUnlocked();
    },
  });
}
