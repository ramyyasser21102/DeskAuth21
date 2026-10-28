import { useState } from "react";
import { restoreVaultBackup, saveVaultBackup } from "@/lib/api";
import { BACKUP_RESTORED, describeBackupSaved } from "./backup-messages";

export interface BackupMessage {
  text: string;
  isError: boolean;
}

/** Runs the backup and restore commands and keeps the sentence saying how they went. */
export function useBackupActions(onVaultChanged: () => void) {
  const [message, setMessage] = useState<BackupMessage | null>(null);

  const report = async (work: () => Promise<string | null>) => {
    try {
      const text = await work();
      setMessage(text === null ? null : { text, isError: false });
    } catch (error) {
      setMessage({ text: String(error), isError: true });
    }
  };

  const saveBackup = () => report(async () => {
    const path = await saveVaultBackup();
    return path === null ? null : describeBackupSaved(path);
  });

  const restoreBackup = () => report(async () => {
    if (!(await restoreVaultBackup())) return null;
    onVaultChanged();
    return BACKUP_RESTORED;
  });

  return { message, saveBackup, restoreBackup, showMessage: (text: string) => setMessage({ text, isError: false }) };
}
