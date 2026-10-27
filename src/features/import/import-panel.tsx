import type { VendorId } from "@/lib/api";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { FilePicker } from "./file-picker";
import { ImportLinkForm } from "./import-link-form";
import { usePasteImages } from "./use-paste-images";
import { useImportAccounts } from "./use-import-accounts";

const PASTE_HINT = "You can also paste a screenshot anywhere on this page.";

export function ImportPanel({ vendor }: { vendor: VendorId }) {
  const { outcome, isBusy, importFromFiles, importFromLink } = useImportAccounts(vendor);
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
