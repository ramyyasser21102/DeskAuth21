import { FolderSync } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useVaultFolder } from "./use-vault-folder";

const SYNC_HINT =
  "To use your vault on several devices, keep it in a folder synced by iCloud Drive, Dropbox, Syncthing or similar. Only the encrypted file is shared.";

export function VaultFolderSetting({ onVaultChanged }: { onVaultChanged: () => void }) {
  const { folder, message, isBusy, choose } = useVaultFolder(onVaultChanged);
  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium">Vault location</span>
      <p className="break-all text-xs text-muted-foreground">{folder ?? "…"}</p>
      <Button type="button" variant="outline" size="sm" disabled={isBusy} onClick={choose} className="self-start">
        <FolderSync /> Choose folder…
      </Button>
      <p className={cn("text-xs text-muted-foreground", message?.isError && "text-destructive")}>
        {message?.text ?? SYNC_HINT}
      </p>
    </div>
  );
}
