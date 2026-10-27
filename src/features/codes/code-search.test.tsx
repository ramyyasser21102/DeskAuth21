import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { CodeSearch } from "./code-search";

afterEach(cleanup);

describe("CodeSearch", () => {
  it("has a visible label and reports what is typed", () => {
    const onChange = vi.fn();
    render(<CodeSearch query="" onChange={onChange} />);
    fireEvent.change(screen.getByLabelText("Search accounts"), { target: { value: "git" } });
    expect(onChange).toHaveBeenCalledWith("git");
    expect(screen.queryByRole("button", { name: "Clear search" })).toBeNull();
  });

  it("clears with the clear button and with Escape", () => {
    const onChange = vi.fn();
    render(<CodeSearch query="git" onChange={onChange} />);
    fireEvent.click(screen.getByRole("button", { name: "Clear search" }));
    fireEvent.keyDown(screen.getByLabelText("Search accounts"), { key: "Escape" });
    expect(onChange).toHaveBeenNthCalledWith(1, "");
    expect(onChange).toHaveBeenNthCalledWith(2, "");
  });
});
