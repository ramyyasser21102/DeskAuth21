import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface ImportLinkFormProps {
  disabled: boolean;
  onImport: (link: string) => Promise<void>;
}

export function ImportLinkForm({ disabled, onImport }: ImportLinkFormProps) {
  const [link, setLink] = useState("");

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    await onImport(link);
    // The link holds the secret, so don't leave it sitting in the field.
    setLink("");
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-2">
      <Label htmlFor="import-link">Paste an otpauth:// link</Label>
      <div className="flex gap-2">
        <Input id="import-link" value={link} autoComplete="off" onChange={(event) => setLink(event.target.value)} />
        <Button type="submit" disabled={disabled || link.trim() === ""}>Import</Button>
      </div>
    </form>
  );
}
