import { Lock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { openAppPage } from "@/lib/api";

export function PanelLocked() {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center">
      <Lock className="size-6 text-muted-foreground" aria-hidden="true" />
      <p className="text-sm font-medium">DeskAuth21 is locked.</p>
      <Button onClick={() => void openAppPage("codes")}>Open DeskAuth21 to unlock</Button>
    </div>
  );
}
