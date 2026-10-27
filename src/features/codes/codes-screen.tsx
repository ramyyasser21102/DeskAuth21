import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ImportPanel } from "@/features/import/import-panel";
import { AutoLockSetting } from "@/features/settings/auto-lock-setting";
import { useActivityReporter } from "@/features/settings/use-activity-reporter";
import { CodeRow } from "./code-row";
import { CodesFooter } from "./codes-footer";
import { useImportVisibility } from "./use-import-visibility";
import { useLiveCodes } from "./use-live-codes";

const COPY_HINT = "Click a code to copy it; it is cleared from the clipboard after 30 seconds.";
const EMPTY_HINT = "No accounts yet. Import them from Google Authenticator below.";

export function CodesScreen({ onLocked }: { onLocked: () => void }) {
  useActivityReporter();
  const { codes, hadNoAccountsOnLoad } = useLiveCodes(onLocked);
  const { isImportOpen, toggleImport } = useImportVisibility(hadNoAccountsOnLoad);

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
        <AutoLockSetting />
      </CardContent>
      <CodesFooter isImportOpen={isImportOpen} onToggleImport={toggleImport} onLocked={onLocked} />
    </Card>
  );
}
