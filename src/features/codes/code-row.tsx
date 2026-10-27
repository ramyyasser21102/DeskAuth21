import { useState } from "react";
import { Pencil } from "lucide-react";
import { IconButton } from "@/components/icon-button";
import type { CodeView } from "@/lib/api";
import { AccountEditor } from "./account-editor";
import { CopyCodeButton } from "./copy-code-button";
import { useFocusReturn } from "./use-focus-return";

export function CodeRow({ view }: { view: CodeView }) {
  const [isEditing, setIsEditing] = useState(false);
  const editButtonRef = useFocusReturn<HTMLButtonElement>(isEditing);
  return (
    <li className="flex items-center gap-1 border-b last:border-b-0">
      {isEditing ? (
        <div className="w-full">
          <AccountEditor view={view} onDone={() => setIsEditing(false)} />
        </div>
      ) : (
        <>
          <CopyCodeButton view={view} />
          <IconButton ref={editButtonRef} label={`Edit ${view.label}`} variant="ghost" onClick={() => setIsEditing(true)}>
            <Pencil />
          </IconButton>
        </>
      )}
    </li>
  );
}
