import { useState, type FormEvent } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { PasswordField } from "@/features/vault/password-field";
import { useRecoveryKeyWizard } from "./use-recovery-key-wizard";

const WRITE_DOWN =
  "Write these 24 words down, in order, and keep them somewhere safe and private. They open your vault if you forget your passphrase, and they are shown only once.";

function WordGrid({ words }: { words: readonly string[] }) {
  return (
    <ol className="grid grid-cols-2 gap-x-6 gap-y-1 rounded-md border p-3 text-sm sm:grid-cols-3" aria-label="Recovery key words">
      {words.map((word, index) => (
        <li key={`${index}-${word}`} className="flex gap-2"><span className="w-5 text-right text-muted-foreground">{index + 1}</span><span className="font-medium">{word}</span></li>
      ))}
    </ol>
  );
}

function ConfirmForm({ wizard, onCancel }: { wizard: ReturnType<typeof useRecoveryKeyWizard>; onCancel: () => void }) {
  const [typed, setTyped] = useState<string[]>(wizard.positions.map(() => ""));
  const [current, setCurrent] = useState("");
  const submit = (event: FormEvent) => {
    event.preventDefault();
    void wizard.save(typed, current);
  };
  return (
    <form onSubmit={submit} className="flex flex-col gap-3">
      <p className="text-sm">Type three of the words to show you wrote them down.</p>
      {wizard.positions.map((position, index) => (
        <div key={position} className="flex flex-col gap-1">
          <Label htmlFor={`recovery-word-${position}`}>{`Word ${position + 1}`}</Label>
          <Input id={`recovery-word-${position}`} autoComplete="off" value={typed[index]} onChange={(event) => setTyped(typed.map((value, at) => (at === index ? event.target.value : value)))} />
        </div>
      ))}
      <PasswordField id="recovery-current-passphrase" label="Current passphrase" value={current} onChange={setCurrent} />
      {wizard.errorMessage && <Alert variant="destructive"><AlertDescription>{wizard.errorMessage}</AlertDescription></Alert>}
      <div className="flex gap-2">
        <Button type="submit" disabled={wizard.isBusy}>Save recovery key</Button>
        <Button type="button" variant="outline" onClick={onCancel}>Cancel</Button>
      </div>
    </form>
  );
}

/** Nothing is saved until the confirm step succeeds, so leaving early changes nothing. */
export function RecoveryKeyWizard({ onClose }: { onClose: () => void }) {
  const wizard = useRecoveryKeyWizard();
  if (wizard.step === "done") {
    return <div className="flex flex-col gap-3"><p role="status" className="text-sm">Recovery key saved. Keep your written copy safe.</p><Button variant="outline" onClick={onClose}>Close</Button></div>;
  }
  if (wizard.step === "confirm") return <ConfirmForm wizard={wizard} onCancel={onClose} />;
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm">{WRITE_DOWN}</p>
      {wizard.words.length > 0 && <WordGrid words={wizard.words} />}
      {wizard.errorMessage && <Alert variant="destructive"><AlertDescription>{wizard.errorMessage}</AlertDescription></Alert>}
      <div className="flex gap-2">
        <Button disabled={wizard.words.length === 0} onClick={wizard.startConfirming}>I have written it down</Button>
        <Button variant="outline" onClick={onClose}>Cancel</Button>
      </div>
    </div>
  );
}
