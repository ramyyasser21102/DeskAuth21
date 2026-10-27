import { describe, expect, it } from "vitest";
import { IMPORT_INSTRUCTIONS } from "./import-instructions";

describe("IMPORT_INSTRUCTIONS", () => {
  it("has non-empty guidance for every vendor", () => {
    for (const text of Object.values(IMPORT_INSTRUCTIONS)) expect(text.length).toBeGreaterThan(20);
  });

  it("tells users of apps without an export to use each service's QR code", () => {
    expect(IMPORT_INSTRUCTIONS.microsoft_authenticator).toContain("has no export");
  });
});
