import { useState } from "react";
import { AppLogo } from "@/components/app-logo";
import { UnlockedApp } from "@/features/pages/unlocked-app";
import { ProfileAvatar } from "@/features/profile/profile-avatar";
import { useProfile } from "@/features/profile/profile-provider";
import { CreateVaultScreen, UnlockScreen } from "@/features/vault/vault-screens";
import { WelcomePage } from "@/features/welcome/welcome-page";
import { hasSeenWelcome, markWelcomeSeen } from "@/features/welcome/welcome-seen";
import type { VaultStatus } from "@/lib/api";
import { useVaultStatus } from "./use-vault-status";

interface ScreenProps {
  status: VaultStatus | null;
  onChange: () => void;
  onShowWelcome: () => void;
}

function ScreenFor({ status, onChange, onShowWelcome }: ScreenProps) {
  switch (status) {
    case "uninitialized":
      return <CreateVaultScreen onCreated={onChange} />;
    case "locked":
      return <UnlockScreen onUnlocked={onChange} />;
    case "unlocked":
      return <UnlockedApp onLocked={onChange} onShowWelcome={onShowWelcome} />;
    case null:
      return null;
  }
}

function AppHeader() {
  const { profile } = useProfile();
  return (
    <header className="mx-auto flex w-full max-w-md items-center justify-between">
      <div className="flex items-center gap-2">
        <AppLogo className="size-7" />
        <span className="text-lg font-semibold tracking-tight">DA21</span>
      </div>
      <ProfileAvatar profile={profile} />
    </header>
  );
}

export function App() {
  const { status, refreshStatus } = useVaultStatus();
  const [isWelcomeOpen, setIsWelcomeOpen] = useState(() => !hasSeenWelcome());
  const closeWelcome = () => {
    markWelcomeSeen();
    setIsWelcomeOpen(false);
  };

  return (
    <div className="flex min-h-screen flex-col gap-6 px-4 py-6">
      <AppHeader />
      <main className="mx-auto w-full max-w-md">
        {isWelcomeOpen && <WelcomePage onContinue={closeWelcome} continueLabel="Start using DA21" />}
        {/* Hidden, not unmounted, so reopening the welcome page keeps your place. */}
        <div hidden={isWelcomeOpen}>
          <ScreenFor status={status} onChange={refreshStatus} onShowWelcome={() => setIsWelcomeOpen(true)} />
        </div>
      </main>
    </div>
  );
}
