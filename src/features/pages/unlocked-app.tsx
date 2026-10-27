import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { useOpenPageRequests } from "@/features/navigation/use-open-page-requests";
import { TabBar } from "@/features/navigation/tab-bar";
import type { PageId } from "@/features/navigation/pages";
import { CaptureBanner } from "@/features/privacy/capture-banner";
import { UpgradePrompt } from "@/features/security/upgrade-prompt";
import { PrivacySettingsProvider } from "@/features/privacy/privacy-settings-provider";
import { useActivityReporter } from "@/features/settings/use-activity-reporter";
import { PAGE_MOTION } from "@/lib/motion-tokens";
import { useLiveCodes } from "@/features/codes/use-live-codes";
import { CodesPage } from "./codes-page";
import { ImportPage } from "./import-page";
import { SettingsPage } from "./settings-page";

/** Everything shown once the vault is open: the current page and the tab bar. */
interface UnlockedAppProps {
  onLocked: () => void;
  onShowWelcome: () => void;
}

export function UnlockedApp({ onLocked, onShowWelcome }: UnlockedAppProps) {
  useActivityReporter();
  const { codes } = useLiveCodes(onLocked);
  const [page, setPage] = useState<PageId>("codes");
  useOpenPageRequests(setPage);

  return (
    <PrivacySettingsProvider>
      <AnimatePresence mode="wait" initial={false}>
        <motion.div key={page} className="pb-20" {...PAGE_MOTION}>
          <UpgradePrompt />
          <CaptureBanner />
          {page === "codes" && <CodesPage codes={codes} onGoToImport={() => setPage("import")} />}
          {page === "import" && <ImportPage />}
          {page === "settings" && <SettingsPage onVaultChanged={onLocked} onShowWelcome={onShowWelcome} />}
        </motion.div>
      </AnimatePresence>
      <TabBar current={page} onSelect={setPage} />
    </PrivacySettingsProvider>
  );
}
