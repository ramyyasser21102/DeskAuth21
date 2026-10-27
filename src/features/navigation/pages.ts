import { Download, KeyRound, Settings, type LucideIcon } from "lucide-react";

export type PageId = "codes" | "import" | "settings";

export interface PageTab {
  id: PageId;
  label: string;
  Icon: LucideIcon;
}

export const PAGE_TABS: readonly PageTab[] = [
  { id: "codes", label: "Codes", Icon: KeyRound },
  { id: "import", label: "Import", Icon: Download },
  { id: "settings", label: "Settings", Icon: Settings },
];
