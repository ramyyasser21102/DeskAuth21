import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { lockVault } from "@/lib/api";
import { CodeRow } from "./code-row";
import { useLiveCodes } from "./use-live-codes";

const IMPORT_HINT = "Import arrives in the next release. Codes also appear in the tray menu.";

export function CodesScreen({ onLocked }: { onLocked: () => void }) {
  const codes = useLiveCodes(onLocked);
  const handleLock = async () => {
    await lockVault();
    onLocked();
  };
  return (
    <Card>
      <CardHeader>
        <CardTitle>Your codes</CardTitle>
        <CardDescription>{IMPORT_HINT}</CardDescription>
      </CardHeader>
      <CardContent>
        <ul>
          {codes.map((view, index) => (
            <CodeRow key={`${index}-${view.label}`} view={view} />
          ))}
        </ul>
      </CardContent>
      <CardFooter>
        <Button variant="outline" onClick={handleLock}>Lock</Button>
      </CardFooter>
    </Card>
  );
}
