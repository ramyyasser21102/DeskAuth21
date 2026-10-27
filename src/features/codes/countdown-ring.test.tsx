import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { CountdownRing } from "./countdown-ring";

afterEach(cleanup);

const progressOffset = (container: HTMLElement) =>
  Number(container.querySelectorAll("circle")[1].getAttribute("stroke-dashoffset"));

describe("CountdownRing", () => {
  it("names the seconds left for screen readers and shows them", () => {
    render(<CountdownRing secondsLeft={12} periodSeconds={30} />);
    expect(screen.getByRole("img", { name: "12 seconds left" })).toBeTruthy();
    expect(screen.getByText("12")).toBeTruthy();
  });

  it("draws less of the ring as the window runs out", () => {
    const { container: early } = render(<CountdownRing secondsLeft={25} periodSeconds={30} />);
    const { container: late } = render(<CountdownRing secondsLeft={5} periodSeconds={30} />);
    expect(progressOffset(late)).toBeGreaterThan(progressOffset(early));
  });

  it("pulses only when about to expire", () => {
    render(<CountdownRing secondsLeft={3} periodSeconds={30} />);
    expect(screen.getByRole("img").className).toContain("animate-pulse");
    cleanup();
    render(<CountdownRing secondsLeft={20} periodSeconds={30} />);
    expect(screen.getByRole("img").className).not.toContain("animate-pulse");
  });
});
