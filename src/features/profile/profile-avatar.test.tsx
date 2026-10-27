import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { ProfileAvatar } from "./profile-avatar";

afterEach(cleanup);

describe("ProfileAvatar", () => {
  it("shows initials without a picture", () => {
    render(<ProfileAvatar profile={{ name: "Ada Lovelace", pictureDataUrl: null }} />);
    expect(screen.getByText("AL")).toBeTruthy();
  });

  it("shows the picture when there is one", () => {
    const { container } = render(<ProfileAvatar profile={{ name: "Ada", pictureDataUrl: "data:image/jpeg;base64,AAAA" }} />);
    expect(container.querySelector("img")?.getAttribute("src")).toBe("data:image/jpeg;base64,AAAA");
    expect(screen.queryByText("A")).toBeNull();
  });
});
