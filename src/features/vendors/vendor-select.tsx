import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { VendorId } from "@/lib/api";
import { useVendors } from "./use-vendors";

interface VendorSelectProps {
  id: string;
  value: VendorId;
  onChange: (vendor: VendorId) => void;
}

export function VendorSelect({ id, value, onChange }: VendorSelectProps) {
  const vendors = useVendors();
  return (
    <Select value={value} onValueChange={(next) => onChange(next as VendorId)}>
      <SelectTrigger id={id} className="w-full">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {vendors.map((vendor) => (
          <SelectItem key={vendor.id} value={vendor.id}>{vendor.name}</SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
