import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { CodeRow } from "@/features/codes/code-row";
import { countByVendor, filterByVendor, resolveFilter, type VendorFilter } from "@/features/vendors/vendor-filter";
import { VendorTabs } from "@/features/vendors/vendor-tabs";
import type { CodeView } from "@/lib/api";

const COPY_HINT = "Click a code to copy it; it is cleared from the clipboard after 30 seconds.";
const EMPTY_HINT = "No accounts yet. Bring them over from another authenticator app.";

interface CodesPageProps {
  /** `null` until the first answer arrives. */
  codes: CodeView[] | null;
  onGoToImport: () => void;
}

export function CodesPage({ codes, onGoToImport }: CodesPageProps) {
  const [chosenFilter, setChosenFilter] = useState<VendorFilter>("all");
  const all = codes ?? [];
  const counts = countByVendor(all);
  const filter = resolveFilter(chosenFilter, counts);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Your codes</CardTitle>
        <CardDescription>{codes?.length === 0 ? EMPTY_HINT : COPY_HINT}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <VendorTabs counts={counts} total={all.length} selected={filter} onSelect={setChosenFilter} />
        <ul>
          {filterByVendor(all, filter).map((view) => (
            <CodeRow key={view.id} view={view} />
          ))}
        </ul>
        {codes?.length === 0 && <Button onClick={onGoToImport}>Import accounts</Button>}
      </CardContent>
    </Card>
  );
}
