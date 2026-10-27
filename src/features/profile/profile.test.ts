import { describe, expect, it } from "vitest";
import { centredSquare, EMPTY_PROFILE, initialsOf, MAX_NAME_LENGTH, parseProfile, serializeProfile } from "./profile";

describe("parseProfile", () => {
  it("round-trips a saved profile", () => {
    const profile = { name: "Ada", pictureDataUrl: "data:image/jpeg;base64,AAAA" };
    expect(parseProfile(serializeProfile(profile))).toEqual(profile);
  });

  it("gives the empty profile for nothing, junk or the wrong shape", () => {
    expect(parseProfile(null)).toEqual(EMPTY_PROFILE);
    expect(parseProfile("{not json")).toEqual(EMPTY_PROFILE);
    expect(parseProfile("null")).toEqual(EMPTY_PROFILE);
    expect(parseProfile('{"name":5,"pictureDataUrl":7}')).toEqual(EMPTY_PROFILE);
  });

  it("refuses a picture that is not an image data URL", () => {
    expect(parseProfile('{"name":"A","pictureDataUrl":"https://example.com/x.png"}').pictureDataUrl).toBeNull();
  });

  it("cuts an overlong name", () => {
    const stored = JSON.stringify({ name: "x".repeat(MAX_NAME_LENGTH + 10) });
    expect(parseProfile(stored).name).toHaveLength(MAX_NAME_LENGTH);
  });
});

describe("initialsOf", () => {
  it("uses the first letters of the first two words", () => {
    expect(initialsOf("ada lovelace")).toBe("AL");
    expect(initialsOf("Ada Augusta King")).toBe("AA");
  });

  it("handles one word, extra spaces and blank", () => {
    expect(initialsOf("  grace ")).toBe("G");
    expect(initialsOf("   ")).toBe("");
  });
});

describe("centredSquare", () => {
  it("crops the sides of a wide image", () => {
    expect(centredSquare(200, 100)).toEqual({ sourceX: 50, sourceY: 0, side: 100 });
  });

  it("crops the top and bottom of a tall image", () => {
    expect(centredSquare(100, 300)).toEqual({ sourceX: 0, sourceY: 100, side: 100 });
  });
});
