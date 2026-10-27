import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { REVEAL_SECONDS } from "@/features/privacy/privacy-state";
import { useReveal } from "./use-reveal";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

const MS = 1000;

describe("useReveal", () => {
  it("starts hidden and toggles", () => {
    const { result } = renderHook(() => useReveal());
    expect(result.current.isRevealed).toBe(false);
    act(() => result.current.toggle());
    expect(result.current.isRevealed).toBe(true);
    act(() => result.current.toggle());
    expect(result.current.isRevealed).toBe(false);
  });

  it("hides again after the reveal time, and not before", () => {
    const { result } = renderHook(() => useReveal());
    act(() => result.current.toggle());
    act(() => vi.advanceTimersByTime((REVEAL_SECONDS - 1) * MS));
    expect(result.current.isRevealed).toBe(true);
    act(() => vi.advanceTimersByTime(MS));
    expect(result.current.isRevealed).toBe(false);
  });

  it("hides at once when the window loses focus", () => {
    const { result } = renderHook(() => useReveal());
    act(() => result.current.toggle());
    act(() => window.dispatchEvent(new Event("blur")));
    expect(result.current.isRevealed).toBe(false);
  });

  it("hides at once when the page becomes hidden", () => {
    const { result } = renderHook(() => useReveal());
    act(() => result.current.toggle());
    act(() => document.dispatchEvent(new Event("visibilitychange")));
    expect(result.current.isRevealed).toBe(false);
  });
});
