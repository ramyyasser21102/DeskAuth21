import { Alert, AlertDescription } from "@/components/ui/alert";
import { ImagePicker } from "./image-picker";
import { ImportLinkForm } from "./import-link-form";
import { usePasteImages } from "./use-paste-images";
import { useImportAccounts } from "./use-import-accounts";

const PASTE_HINT =
  "In Google Authenticator choose Transfer accounts, then Export. Screenshot each QR code and choose or paste the images here, or paste the link.";

export function ImportPanel() {
  const { outcome, isBusy, importFromImages, importFromLink } = useImportAccounts();
  usePasteImages(importFromImages);

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-muted-foreground">{PASTE_HINT}</p>
      <ImagePicker disabled={isBusy} onFiles={importFromImages} />
      <ImportLinkForm disabled={isBusy} onImport={importFromLink} />
      {outcome && (
        <Alert variant={outcome.isError ? "destructive" : "default"} role="status">
          <AlertDescription>{outcome.message}</AlertDescription>
        </Alert>
      )}
    </div>
  );
}
