import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { SecuritySettings } from "@/features/security/security-settings";
import { PrivacySettings } from "@/features/privacy/privacy-settings";
import { ProfileSetting } from "@/features/profile/profile-setting";
import { ShortcutSettings } from "@/features/shortcuts/shortcut-settings";
import { ThemeToggle } from "@/features/theme/theme-toggle";
import { AutoLockSetting } from "@/features/settings/auto-lock-setting";
import { VaultFolderSetting } from "@/features/sync/vault-folder-setting";
import { lockVault } from "@/lib/api";

interface SettingsPageProps {
  onVaultChanged: () => void;
  onShowWelcome: () => void;
}

export function SettingsPage({ onVaultChanged, onShowWelcome }: SettingsPageProps) {
  const handleLock = async () => {
    await lockVault();
    onVaultChanged();
  };
  return (
    <Card>
      <CardHeader>
        <CardTitle>Settings</CardTitle>
        <CardDescription>You, your privacy, how it looks, how it locks and where it keeps your vault.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-6">
        <ProfileSetting />
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">Appearance</span>
          <ThemeToggle />
        </div>
        <PrivacySettings />
        <ShortcutSettings />
        <SecuritySettings />
        <AutoLockSetting />
        <VaultFolderSetting onVaultChanged={onVaultChanged} />
        <Button variant="outline" onClick={onShowWelcome}>Show welcome page</Button>
        <Button variant="outline" onClick={handleLock}>Lock now</Button>
      </CardContent>
    </Card>
  );
}
