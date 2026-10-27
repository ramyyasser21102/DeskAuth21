import { useRef, type ChangeEvent } from "react";
import { ImageUp } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ImagePickerProps {
  disabled: boolean;
  onFiles: (files: File[]) => void;
}

export function ImagePicker({ disabled, onFiles }: ImagePickerProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    onFiles(Array.from(event.target.files ?? []));
    // Reset so choosing the same file again still fires a change.
    event.target.value = "";
  };

  return (
    <div className="flex flex-col gap-2">
      <Button type="button" variant="outline" disabled={disabled} onClick={() => inputRef.current?.click()}>
        <ImageUp /> Choose QR images
      </Button>
      <input ref={inputRef} type="file" accept="image/*" multiple hidden onChange={handleChange} aria-label="QR code images" />
    </div>
  );
}
