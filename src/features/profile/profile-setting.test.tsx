import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { parseProfile } from "./profile";
import { PROFILE_STORAGE_KEY, ProfileProvider } from "./profile-provider";
import { ProfileSetting } from "./profile-setting";

afterEach(() => {
  cleanup();
  window.localStorage.clear();
});

const savedProfile = () => parseProfile(window.localStorage.getItem(PROFILE_STORAGE_KEY));

describe("ProfileSetting", () => {
  it("saves the name on this device as it is typed", () => {
    render(<ProfileProvider><ProfileSetting /></ProfileProvider>);
    fireEvent.change(screen.getByLabelText("Your name"), { target: { value: "Ada Lovelace" } });
    expect(savedProfile().name).toBe("Ada Lovelace");
    expect(screen.getByText("AL")).toBeTruthy();
  });

  it("starts from what was saved and offers to remove a picture", () => {
    window.localStorage.setItem(PROFILE_STORAGE_KEY, JSON.stringify({ name: "Grace", pictureDataUrl: "data:image/jpeg;base64,AAAA" }));
    render(<ProfileProvider><ProfileSetting /></ProfileProvider>);
    expect((screen.getByLabelText("Your name") as HTMLInputElement).value).toBe("Grace");
    fireEvent.click(screen.getByRole("button", { name: "Remove" }));
    expect(savedProfile().pictureDataUrl).toBeNull();
    expect(screen.queryByRole("button", { name: "Remove" })).toBeNull();
  });
});
