import { VaultFolderSetting } from "@/features/sync/vault-folder-setting";
import { AutoLockSetting } from "./auto-lock-setting";

/** Everything the user can configure, kept together and separate from the code list. */
export function SettingsSection({ onVaultChanged }: { onVaultChanged: () => void }) {
  return (
    <div className="flex flex-col gap-6">
      <AutoLockSetting />
      <VaultFolderSetting onVaultChanged={onVaultChanged} />
    </div>
  );
}
