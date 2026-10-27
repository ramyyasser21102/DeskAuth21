import { describe, expect, it, vi } from "vitest";
import { throttle } from "./throttle";

describe("throttle", () => {
  it("runs the first call immediately and ignores calls inside the interval", () => {
    let time = 0;
    const action = vi.fn();
    const throttled = throttle(action, 1000, () => time);
    throttled();
    time = 999;
    throttled();
    expect(action).toHaveBeenCalledTimes(1);
  });

  it("runs again once the interval has passed", () => {
    let time = 0;
    const action = vi.fn();
    const throttled = throttle(action, 1000, () => time);
    throttled();
    time = 1000;
    throttled();
    expect(action).toHaveBeenCalledTimes(2);
  });
});
