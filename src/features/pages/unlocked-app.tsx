import { useState } from "react";
import { TabBar } from "@/features/navigation/tab-bar";
import type { PageId } from "@/features/navigation/pages";
import { useActivityReporter } from "@/features/settings/use-activity-reporter";
import { useLiveCodes } from "@/features/codes/use-live-codes";
import { CodesPage } from "./codes-page";
import { ImportPage } from "./import-page";
import { SettingsPage } from "./settings-page";

/** Everything shown once the vault is open: the current page and the tab bar. */
export function UnlockedApp({ onLocked }: { onLocked: () => void }) {
  useActivityReporter();
  const { codes } = useLiveCodes(onLocked);
  const [page, setPage] = useState<PageId>("codes");

  return (
    <>
      <div className="pb-20">
        {page === "codes" && <CodesPage codes={codes} onGoToImport={() => setPage("import")} />}
        {page === "import" && <ImportPage />}
        {page === "settings" && <SettingsPage onVaultChanged={onLocked} />}
      </div>
      <TabBar current={page} onSelect={setPage} />
    </>
  );
}
