import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { ImportPanel } from "@/features/import/import-panel";
import { IMPORT_INSTRUCTIONS } from "@/features/vendors/import-instructions";
import { VendorSelect } from "@/features/vendors/vendor-select";
import type { VendorId } from "@/lib/api";

const VENDOR_SELECT_ID = "import-vendor";

export function ImportPage() {
  const [vendor, setVendor] = useState<VendorId>("other");
  return (
    <Card>
      <CardHeader>
        <CardTitle>Import accounts</CardTitle>
        <CardDescription>Pick the app your accounts are in now; they are tagged with it.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <Label htmlFor={VENDOR_SELECT_ID}>Import from</Label>
          <VendorSelect id={VENDOR_SELECT_ID} value={vendor} onChange={setVendor} />
          <p className="text-sm text-muted-foreground">{IMPORT_INSTRUCTIONS[vendor]}</p>
        </div>
        <ImportPanel vendor={vendor} />
      </CardContent>
    </Card>
  );
}
