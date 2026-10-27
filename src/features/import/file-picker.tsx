import { useRef, type ChangeEvent } from "react";
import { ImageUp } from "lucide-react";
import { Button } from "@/components/ui/button";

/** QR code images, and the export files other apps produce. */
const ACCEPTED_FILES = "image/*,.json,.csv,.txt,application/json,text/csv,text/plain";

interface FilePickerProps {
  disabled: boolean;
  onFiles: (files: File[]) => void;
}

export function FilePicker({ disabled, onFiles }: FilePickerProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    onFiles(Array.from(event.target.files ?? []));
    // Reset so choosing the same file again still fires a change.
    event.target.value = "";
  };

  return (
    <div className="flex flex-col gap-2">
      <Button type="button" variant="outline" disabled={disabled} onClick={() => inputRef.current?.click()}>
        <ImageUp /> Choose files…
      </Button>
      <input
        ref={inputRef}
        type="file"
        accept={ACCEPTED_FILES}
        multiple
        hidden
        onChange={handleChange}
        aria-label="Files to import"
      />
    </div>
  );
}
