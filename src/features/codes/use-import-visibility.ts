import { useState } from "react";

/**
 * Whether the import panel is shown. It starts open only if the vault was empty
 * when first loaded, and from then on stays as it is until the user toggles it,
 * so importing the first account does not make the panel vanish mid-import.
 */
export function useImportVisibility(hadNoAccountsOnLoad: boolean | null) {
  const [userChoice, setUserChoice] = useState<boolean | null>(null);
  const isImportOpen = userChoice ?? hadNoAccountsOnLoad === true;
  const toggleImport = () => setUserChoice(!isImportOpen);
  return { isImportOpen, toggleImport };
}
