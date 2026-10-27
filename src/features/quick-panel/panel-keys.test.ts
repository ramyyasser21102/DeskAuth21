import { describe, expect, it } from "vitest";
import { isPanelView, nextRowIndex } from "./panel-keys";

describe("nextRowIndex", () => {
  it("moves down from the search box to the first row and stops at the last", () => {
    expect(nextRowIndex(-1, "ArrowDown", 3)).toBe(0);
    expect(nextRowIndex(1, "ArrowDown", 3)).toBe(2);
    expect(nextRowIndex(2, "ArrowDown", 3)).toBe(2);
  });

  it("moves up and returns to the search box above the first row", () => {
    expect(nextRowIndex(2, "ArrowUp", 3)).toBe(1);
    expect(nextRowIndex(0, "ArrowUp", 3)).toBe(-1);
  });

  it("stays in the search box when there are no rows", () => {
    expect(nextRowIndex(-1, "ArrowDown", 0)).toBe(-1);
  });
});

describe("isPanelView", () => {
  it("is true only for the panel's address", () => {
    expect(isPanelView("?view=panel")).toBe(true);
    expect(isPanelView("")).toBe(false);
    expect(isPanelView("?view=other")).toBe(false);
  });
});
