import { UnlockedApp } from "@/features/pages/unlocked-app";
import { ThemeToggle } from "@/features/theme/theme-toggle";
import { CreateVaultScreen, UnlockScreen } from "@/features/vault/vault-screens";
import type { VaultStatus } from "@/lib/api";
import { useVaultStatus } from "./use-vault-status";

function ScreenFor({ status, onChange }: { status: VaultStatus | null; onChange: () => void }) {
  switch (status) {
    case "uninitialized":
      return <CreateVaultScreen onCreated={onChange} />;
    case "locked":
      return <UnlockScreen onUnlocked={onChange} />;
    case "unlocked":
      return <UnlockedApp onLocked={onChange} />;
    case null:
      return null;
  }
}

export function App() {
  const { status, refreshStatus } = useVaultStatus();
  return (
    <div className="mx-auto flex min-h-screen max-w-md flex-col gap-6 px-4 py-6">
      <header className="flex items-center justify-between">
        <h1 className="text-lg font-semibold tracking-tight">DeskAuth21</h1>
        <ThemeToggle />
      </header>
      <main>
        <ScreenFor status={status} onChange={refreshStatus} />
      </main>
    </div>
  );
}
