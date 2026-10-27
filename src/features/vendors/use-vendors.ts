import { useEffect, useState } from "react";
import { getVendors, type VendorId, type VendorInfo } from "@/lib/api";

/** The vendor catalogue from the backend; empty until it loads. */
export function useVendors(): VendorInfo[] {
  const [vendors, setVendors] = useState<VendorInfo[]>([]);
  useEffect(() => {
    let isActive = true;
    getVendors()
      .then((catalog) => isActive && setVendors(catalog))
      .catch(() => isActive && setVendors([]));
    return () => {
      isActive = false;
    };
  }, []);
  return vendors;
}

export function vendorName(vendors: readonly VendorInfo[], id: VendorId): string {
  return vendors.find((vendor) => vendor.id === id)?.name ?? id;
}
