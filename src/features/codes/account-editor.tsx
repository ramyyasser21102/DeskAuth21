import type { CodeView } from "@/lib/api";
import { DeleteConfirmation } from "./delete-confirmation";
import { LabelsForm } from "./labels-form";
import { useAccountEditor } from "./use-account-editor";

interface AccountEditorProps {
  view: CodeView;
  onDone: () => void;
}

export function AccountEditor({ view, onDone }: AccountEditorProps) {
  const editor = useAccountEditor(view, onDone);
  if (editor.step === "confirming-delete") {
    return (
      <DeleteConfirmation
        accountLabel={view.label}
        isBusy={editor.isBusy}
        onConfirm={editor.confirmDelete}
        onCancel={editor.cancelDelete}
      />
    );
  }
  return <LabelsForm view={view} editor={editor} onCancel={onDone} />;
}
