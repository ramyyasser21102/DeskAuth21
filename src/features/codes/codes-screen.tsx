import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { ImportPanel } from "@/features/import/import-panel";
import { lockVault } from "@/lib/api";
import { CodeRow } from "./code-row";
import { useImportVisibility } from "./use-import-visibility";
import { useLiveCodes } from "./use-live-codes";

const COPY_HINT = "Click a code to copy it; it is cleared from the clipboard after 30 seconds.";
const EMPTY_HINT = "No accounts yet. Import them from Google Authenticator below.";

export function CodesScreen({ onLocked }: { onLocked: () => void }) {
  const { codes, hadNoAccountsOnLoad } = useLiveCodes(onLocked);
  const { isImportOpen, toggleImport } = useImportVisibility(hadNoAccountsOnLoad);

  const handleLock = async () => {
    await lockVault();
    onLocked();
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Your codes</CardTitle>
        <CardDescription>{codes?.length === 0 ? EMPTY_HINT : COPY_HINT}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-6">
        <ul>
          {codes?.map((view, index) => (
            <CodeRow key={`${index}-${view.label}`} view={view} accountIndex={index} />
          ))}
        </ul>
        {isImportOpen && <ImportPanel />}
      </CardContent>
      <CardFooter className="gap-2">
        <Button variant="outline" onClick={toggleImport}>
          {isImportOpen ? "Hide import" : "Add accounts"}
        </Button>
        <Button variant="outline" onClick={handleLock}>Lock</Button>
      </CardFooter>
    </Card>
  );
}
