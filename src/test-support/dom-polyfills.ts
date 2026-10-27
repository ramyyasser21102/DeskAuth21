import type { ReactNode } from "react";
import { vi } from "vitest";

// jsdom lacks ResizeObserver, which Radix uses to place tooltips and select menus.
class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}

globalThis.ResizeObserver ??= ResizeObserverStub;

// Exit animations would keep removed elements in the page while a test looks for them,
// so tests see content change at once. The animations themselves are not under test.
vi.mock("motion/react", async (importOriginal) => ({
  ...(await importOriginal<typeof import("motion/react")>()),
  AnimatePresence: ({ children }: { children?: ReactNode }) => children,
}));
