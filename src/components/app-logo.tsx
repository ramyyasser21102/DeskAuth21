import appIconUrl from "../../assets/app-icon.svg";
import { cn } from "@/lib/utils";

/** The same artwork as the app icon, so the window and the Dock agree. */
export function AppLogo({ className }: { className?: string }) {
  return <img src={appIconUrl} alt="DeskAuth21 logo" className={cn("size-10", className)} draggable={false} />;
}
