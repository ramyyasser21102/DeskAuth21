/** Where arrow keys move to in a list of `count` rows. Past the first row is the search box (-1). */
export function nextRowIndex(current: number, key: "ArrowDown" | "ArrowUp", count: number): number {
  if (count === 0) return -1;
  if (key === "ArrowDown") return Math.min(current + 1, count - 1);
  return current <= 0 ? -1 : current - 1;
}

export const isPanelView = (search: string): boolean => new URLSearchParams(search).get("view") === "panel";
