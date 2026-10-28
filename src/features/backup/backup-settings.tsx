import { useState } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { ExportAccountsForm } from "./export-accounts-form";
import { RestoreConfirmation } from "./restore-confirmation";
import { useBackupActions } from "./use-backup-actions";

type Step = "buttons" | "restore" | "export";

const HINT = "A backup is a copy of your encrypted vault. Keep it anywhere; it still needs your passphrase.";

export function BackupSettings({ onVaultChanged }: { onVaultChanged: () => void }) {
  const [step, setStep] = useState<Step>("buttons");
  const { message, saveBackup, restoreBackup, showMessage } = useBackupActions(onVaultChanged);
  const backToButtons = () => setStep("buttons");

  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium">Backup and export</span>
      {step === "restore" && <RestoreConfirmation onCancel={backToButtons} onConfirm={() => { backToButtons(); void restoreBackup(); }} />}
      {step === "export" && <ExportAccountsForm onCancel={backToButtons} onDone={(text) => { backToButtons(); showMessage(text); }} />}
      {step === "buttons" && (
        <div className="flex flex-wrap gap-2">
          <Button type="button" variant="outline" size="sm" onClick={() => void saveBackup()}>Save a backup…</Button>
          <Button type="button" variant="outline" size="sm" onClick={() => setStep("restore")}>Restore a backup…</Button>
          <Button type="button" variant="outline" size="sm" onClick={() => setStep("export")}>Export accounts…</Button>
        </div>
      )}
      <p role="status" className={cn("text-xs text-muted-foreground", message?.isError && "text-destructive")}>{message?.text ?? HINT}</p>
    </div>
  );
}
