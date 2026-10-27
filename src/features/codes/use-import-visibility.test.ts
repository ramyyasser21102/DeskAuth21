import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useImportVisibility } from "./use-import-visibility";

describe("useImportVisibility", () => {
  it("stays closed while the first load is pending", () => {
    const { result } = renderHook(() => useImportVisibility(null));
    expect(result.current.isImportOpen).toBe(false);
  });

  it("opens when the vault was empty on load, and closes when it was not", () => {
    expect(renderHook(() => useImportVisibility(true)).result.current.isImportOpen).toBe(true);
    expect(renderHook(() => useImportVisibility(false)).result.current.isImportOpen).toBe(false);
  });

  it("lets the user override the automatic choice", () => {
    const { result } = renderHook(() => useImportVisibility(true));
    act(() => result.current.toggleImport());
    expect(result.current.isImportOpen).toBe(false);
    act(() => result.current.toggleImport());
    expect(result.current.isImportOpen).toBe(true);
  });
});
