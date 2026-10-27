import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { AutoLockSetting } from "@/features/settings/auto-lock-setting";
import { VaultFolderSetting } from "@/features/sync/vault-folder-setting";
import { lockVault } from "@/lib/api";

interface SettingsPageProps {
  onVaultChanged: () => void;
}

export function SettingsPage({ onVaultChanged }: SettingsPageProps) {
  const handleLock = async () => {
    await lockVault();
    onVaultChanged();
  };
  return (
    <Card>
      <CardHeader>
        <CardTitle>Settings</CardTitle>
        <CardDescription>How DeskAuth21 locks and where it keeps your vault.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-6">
        <AutoLockSetting />
        <VaultFolderSetting onVaultChanged={onVaultChanged} />
        <Button variant="outline" onClick={handleLock}>Lock now</Button>
      </CardContent>
    </Card>
  );
}
