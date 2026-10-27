import { useState } from "react";
import { Pencil } from "lucide-react";
import { IconButton } from "@/components/icon-button";
import type { CodeView } from "@/lib/api";
import { AccountEditor } from "./account-editor";
import { CopyCodeButton } from "./copy-code-button";
import { usePrivacySettings } from "@/features/privacy/privacy-settings-provider";
import { RevealButton } from "./reveal-button";
import { useFocusReturn } from "./use-focus-return";
import { useReveal } from "./use-reveal";

export function CodeRow({ view }: { view: CodeView }) {
  const [isEditing, setIsEditing] = useState(false);
  const editButtonRef = useFocusReturn<HTMLButtonElement>(isEditing);
  const { settings } = usePrivacySettings();
  const { isRevealed, toggle } = useReveal();
  // Until the settings load, hide: showing digits is the mistake that cannot be undone.
  const areCodesHidden = settings?.hide_codes ?? true;
  return (
    <li className="flex items-center gap-1 border-b last:border-b-0">
      {isEditing ? (
        <div className="w-full">
          <AccountEditor view={view} onDone={() => setIsEditing(false)} />
        </div>
      ) : (
        <>
          <CopyCodeButton view={view} isCodeHidden={areCodesHidden && !isRevealed} />
          {areCodesHidden && <RevealButton accountLabel={view.label} isRevealed={isRevealed} onToggle={toggle} />}
          <IconButton ref={editButtonRef} label={`Edit ${view.label}`} variant="ghost" onClick={() => setIsEditing(true)}>
            <Pencil />
          </IconButton>
        </>
      )}
    </li>
  );
}
