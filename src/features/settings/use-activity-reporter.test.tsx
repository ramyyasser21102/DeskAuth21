import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { recordActivity } from "@/lib/api";
import { useActivityReporter } from "./use-activity-reporter";

vi.mock("@/lib/api", () => ({ recordActivity: vi.fn().mockResolvedValue(undefined) }));

function Reporter() {
  useActivityReporter();
  return null;
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("useActivityReporter", () => {
  it("reports user input, but only once for a burst of events", () => {
    render(<Reporter />);
    fireEvent.pointerDown(document);
    fireEvent.keyDown(document, { key: "a" });
    fireEvent.pointerMove(document);
    expect(recordActivity).toHaveBeenCalledTimes(1);
  });

  it("stops listening when the screen goes away", () => {
    const { unmount } = render(<Reporter />);
    unmount();
    fireEvent.pointerDown(document);
    expect(recordActivity).not.toHaveBeenCalled();
  });

  it("does not report anything just for being mounted", () => {
    render(<Reporter />);
    expect(recordActivity).not.toHaveBeenCalled();
  });
});
