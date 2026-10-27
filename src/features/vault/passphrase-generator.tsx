import { useState } from "react";
import { Button } from "@/components/ui/button";
import { generatePassphrase } from "@/lib/api";

interface PassphraseGeneratorProps {
  onGenerated: (passphrase: string) => void;
}

/** Makes a passphrase and shows it, because the person must write it down: it cannot be recovered. */
export function PassphraseGenerator({ onGenerated }: PassphraseGeneratorProps) {
  const [generated, setGenerated] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const generate = async () => {
    try {
      const phrase = await generatePassphrase();
      setGenerated(phrase);
      setProblem(null);
      onGenerated(phrase);
    } catch (error) {
      setProblem(String(error));
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <Button type="button" variant="outline" onClick={() => void generate()}>Generate a passphrase</Button>
      {generated && (
        <div className="flex flex-col gap-1 rounded-md border p-3" role="status">
          <code className="break-words text-sm font-medium">{generated}</code>
          <p className="text-xs text-muted-foreground">Written into both fields. Write it down somewhere safe: nobody can recover it for you.</p>
        </div>
      )}
      {problem && <p role="alert" className="text-xs text-destructive">{problem}</p>}
    </div>
  );
}
