import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { TabBar } from "./tab-bar";

afterEach(cleanup);

describe("TabBar", () => {
  it("offers Codes, Import and Settings and marks the current page", () => {
    render(<TabBar current="import" onSelect={() => {}} />);
    expect(screen.getByRole("button", { name: "Import" }).getAttribute("aria-current")).toBe("page");
    expect(screen.getByRole("button", { name: "Codes" }).getAttribute("aria-current")).toBeNull();
    expect(screen.getByRole("button", { name: "Settings" })).toBeTruthy();
  });

  it("reports the page that was tapped", () => {
    const onSelect = vi.fn();
    render(<TabBar current="codes" onSelect={onSelect} />);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(onSelect).toHaveBeenCalledWith("settings");
  });
});
