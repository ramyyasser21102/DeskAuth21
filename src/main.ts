import { renderCodesScreen } from "./features/codes/codes-screen";
import { renderCreateVaultScreen, renderUnlockScreen } from "./features/vault/vault-screens";
import { getVaultStatus, VaultStatus } from "./lib/api";

const app = document.querySelector<HTMLElement>("#app")!;
let stopCurrentScreen: () => void = () => {};

function renderScreenFor(status: VaultStatus): () => void {
  switch (status) {
    case "uninitialized":
      renderCreateVaultScreen(app, showCurrentScreen);
      return () => {};
    case "locked":
      renderUnlockScreen(app, showCurrentScreen);
      return () => {};
    case "unlocked":
      return renderCodesScreen(app, showCurrentScreen);
  }
}

async function showCurrentScreen(): Promise<void> {
  stopCurrentScreen();
  stopCurrentScreen = renderScreenFor(await getVaultStatus());
}

void showCurrentScreen();
