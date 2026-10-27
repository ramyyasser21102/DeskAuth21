import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { hasSeenWelcome, markWelcomeSeen, WELCOME_SEEN_KEY } from "./welcome-seen";
import { WelcomePage } from "./welcome-page";

afterEach(() => {
  cleanup();
  window.localStorage.clear();
});

describe("WelcomePage", () => {
  it("explains what the app does and how it keeps codes safe", () => {
    render(<WelcomePage onContinue={() => {}} continueLabel="Get started" />);
    expect(screen.getByRole("img", { name: "DeskAuth21 logo" })).toBeTruthy();
    expect(screen.getByText("How it works")).toBeTruthy();
    expect(screen.getByText("How your codes stay safe")).toBeTruthy();
    expect(screen.getByText(/Your passphrase is never saved/)).toBeTruthy();
  });

  it("continues when the button is pressed", () => {
    const onContinue = vi.fn();
    render(<WelcomePage onContinue={onContinue} continueLabel="Get started" />);
    fireEvent.click(screen.getByRole("button", { name: "Get started" }));
    expect(onContinue).toHaveBeenCalledOnce();
  });
});

describe("welcome-seen", () => {
  it("is unseen until marked, then stays seen", () => {
    expect(hasSeenWelcome()).toBe(false);
    markWelcomeSeen();
    expect(hasSeenWelcome()).toBe(true);
    expect(window.localStorage.getItem(WELCOME_SEEN_KEY)).toBe("yes");
  });
});
