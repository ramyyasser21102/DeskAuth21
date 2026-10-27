import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useOpenPageRequests } from "./use-open-page-requests";

type Handler = (event: { payload: unknown }) => void;
let handler: Handler | null = null;
const stop = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, callback: Handler) => {
    if (name === "open-page") handler = callback;
    return stop;
  }),
}));

beforeEach(() => {
  handler = null;
  (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
});
afterEach(() => {
  cleanup();
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  vi.clearAllMocks();
});

const settle = () => act(async () => {});

describe("useOpenPageRequests", () => {
  it("passes on a page the panel asks for", async () => {
    const onPage = vi.fn();
    renderHook(() => useOpenPageRequests(onPage));
    await settle();
    handler?.({ payload: "settings" });
    expect(onPage).toHaveBeenCalledWith("settings");
  });

  it("ignores anything that is not one of the pages", async () => {
    const onPage = vi.fn();
    renderHook(() => useOpenPageRequests(onPage));
    await settle();
    handler?.({ payload: "somewhere-else" });
    handler?.({ payload: 5 });
    expect(onPage).not.toHaveBeenCalled();
  });

  it("stops listening when the window goes away", async () => {
    const { unmount } = renderHook(() => useOpenPageRequests(vi.fn()));
    await settle();
    unmount();
    expect(stop).toHaveBeenCalled();
  });

  it("does nothing outside the real app", async () => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    renderHook(() => useOpenPageRequests(vi.fn()));
    await settle();
    expect(handler).toBeNull();
  });
});
