import { ShieldAlert } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { setCaptureProtection, setHideCodes, type Settings } from "@/lib/api";
import { bannerContentFor, type BannerAction } from "./privacy-state";
import { usePrivacySettings } from "./privacy-settings-provider";

const ACTIONS: Record<BannerAction, { label: string; run: () => Promise<Settings> }> = {
  "turn-on-protection": { label: "Turn on protection", run: () => setCaptureProtection(true) },
  "hide-codes": { label: "Hide codes", run: () => setHideCodes(true) },
};

/** Stays on screen for as long as the codes could be captured; it cannot be dismissed. */
export function CaptureBanner() {
  const { settings, applySaved } = usePrivacySettings();
  const content = settings && bannerContentFor(settings);
  if (!content) return null;
  const action = content.action && ACTIONS[content.action];

  return (
    <Alert variant="destructive" role="alert" className="mb-4">
      <ShieldAlert aria-hidden="true" />
      <AlertTitle>{content.title}</AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-2">
        {content.note}
        {action && <Button size="sm" onClick={() => void action.run().then(applySaved)}>{action.label}</Button>}
      </AlertDescription>
    </Alert>
  );
}
