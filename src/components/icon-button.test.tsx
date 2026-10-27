import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { IconButton } from "./icon-button";

afterEach(cleanup);

describe("IconButton", () => {
  it("is named for screen readers and runs its action", () => {
    const onClick = vi.fn();
    render(<IconButton label="Edit Acme" onClick={onClick}><span /></IconButton>);
    fireEvent.click(screen.getByRole("button", { name: "Edit Acme" }));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("shows a real tooltip with the same name when it receives keyboard focus", async () => {
    render(<IconButton label="Edit Acme"><span /></IconButton>);
    fireEvent.focus(screen.getByRole("button", { name: "Edit Acme" }));
    expect((await screen.findAllByText("Edit Acme")).length).toBeGreaterThan(0);
    expect(document.querySelector("[data-slot=tooltip-content]")).toBeTruthy();
  });
});
