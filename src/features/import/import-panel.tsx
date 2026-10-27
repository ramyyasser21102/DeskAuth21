import { Alert, AlertDescription } from "@/components/ui/alert";
import { FilePicker } from "./file-picker";
import { ImportLinkForm } from "./import-link-form";
import { usePasteImages } from "./use-paste-images";
import { useImportAccounts } from "./use-import-accounts";

const PASTE_HINT =
  "Choose QR code images (for Google Authenticator: Transfer accounts, then Export), or an export file from Aegis, 2FAS or Bitwarden. You can also paste a screenshot or an otpauth link.";

export function ImportPanel() {
  const { outcome, isBusy, importFromFiles, importFromLink } = useImportAccounts();
  usePasteImages(importFromFiles);

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-muted-foreground">{PASTE_HINT}</p>
      <FilePicker disabled={isBusy} onFiles={importFromFiles} />
      <ImportLinkForm disabled={isBusy} onImport={importFromLink} />
      {outcome && (
        <Alert variant={outcome.isError ? "destructive" : "default"} role="status">
          <AlertDescription>{outcome.message}</AlertDescription>
        </Alert>
      )}
    </div>
  );
}
