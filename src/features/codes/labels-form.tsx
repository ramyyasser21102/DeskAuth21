import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { VendorSelect } from "@/features/vendors/vendor-select";
import type { CodeView } from "@/lib/api";
import type { useAccountEditor } from "./use-account-editor";

interface LabelsFormProps {
  view: CodeView;
  editor: ReturnType<typeof useAccountEditor>;
  onCancel: () => void;
}

/** Edit an account's service and name, with an escape hatch to deleting it. */
export function LabelsForm({ view, editor, onCancel }: LabelsFormProps) {
  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    void editor.save();
  };
  return (
    <form className="flex flex-col gap-3 py-3" onSubmit={handleSubmit}>
      <Label htmlFor={`issuer-${view.id}`}>Service</Label>
      <Input id={`issuer-${view.id}`} value={editor.issuer} onChange={(event) => editor.setIssuer(event.target.value)} />
      <Label htmlFor={`name-${view.id}`}>Account name</Label>
      <Input id={`name-${view.id}`} value={editor.name} onChange={(event) => editor.setName(event.target.value)} />
      <Label htmlFor={`vendor-${view.id}`}>App</Label>
      <VendorSelect id={`vendor-${view.id}`} value={editor.vendor} onChange={editor.setVendor} />
      {editor.errorMessage && (
        <Alert variant="destructive">
          <AlertDescription>{editor.errorMessage}</AlertDescription>
        </Alert>
      )}
      <div className="flex gap-2">
        <Button type="submit" disabled={editor.isBusy}>Save</Button>
        <Button type="button" variant="outline" onClick={onCancel}>Cancel</Button>
        <Button type="button" variant="ghost" className="ml-auto text-destructive" onClick={editor.askToDelete}>Delete…</Button>
      </div>
    </form>
  );
}
