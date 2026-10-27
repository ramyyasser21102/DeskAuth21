import { useState } from "react";
import { Pencil } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { CodeView } from "@/lib/api";
import { AccountEditor } from "./account-editor";
import { CopyCodeButton } from "./copy-code-button";

export function CodeRow({ view }: { view: CodeView }) {
  const [isEditing, setIsEditing] = useState(false);
  return (
    <li className="flex items-center gap-1 border-b last:border-b-0">
      {isEditing ? (
        <div className="w-full">
          <AccountEditor view={view} onDone={() => setIsEditing(false)} />
        </div>
      ) : (
        <>
          <CopyCodeButton view={view} />
          <Button type="button" size="icon-sm" variant="ghost" aria-label={`Edit ${view.label}`} onClick={() => setIsEditing(true)}>
            <Pencil />
          </Button>
        </>
      )}
    </li>
  );
}
