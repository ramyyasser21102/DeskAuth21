export const describeBackupSaved = (path: string): string =>
  `Backup saved to ${path}. It is encrypted: it opens with the passphrase you use now.`;

export const BACKUP_RESTORED =
  "Backup restored and the vault locked. Unlock it with the passphrase the backup was made with. The vault it replaced was kept next to it.";

export const describeExportResult = (count: number): string =>
  `Exported ${count} ${count === 1 ? "account" : "accounts"}. The file is not encrypted; keep it private.`;
