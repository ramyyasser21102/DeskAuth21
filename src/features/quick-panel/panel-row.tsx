import { CopyCodeButton } from "@/features/codes/copy-code-button";
import { RevealButton } from "@/features/codes/reveal-button";
import { useReveal } from "@/features/codes/use-reveal";
import { usePrivacySettings } from "@/features/privacy/privacy-settings-provider";
import type { CodeView } from "@/lib/api";

interface PanelRowProps {
  view: CodeView;
  onCopied: () => void;
}

/** One account: click to copy, with the same hidden-until-revealed rule as the window. */
export function PanelRow({ view, onCopied }: PanelRowProps) {
  const { settings } = usePrivacySettings();
  const { isRevealed, toggle } = useReveal();
  // Until the settings load, hide: showing digits is the mistake that cannot be undone.
  const areCodesHidden = settings?.hide_codes ?? true;
  return (
    <li className="flex items-center gap-1">
      <CopyCodeButton view={view} isCodeHidden={areCodesHidden && !isRevealed} onCopied={onCopied} wrapLabel />
      {areCodesHidden && <RevealButton accountLabel={view.label} isRevealed={isRevealed} onToggle={toggle} />}
    </li>
  );
}
