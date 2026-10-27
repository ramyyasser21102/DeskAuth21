import { describe, expect, it } from "vitest";
import type { RelocationOutcome } from "@/lib/api";
import { describeRelocation, requiresUnlockAgain } from "./relocation-message";

const ALL_OUTCOMES: RelocationOutcome[] = ["unchanged", "moved", "used_existing", "selected"];

describe("describeRelocation", () => {
  it("has a distinct message for every outcome", () => {
    const messages = ALL_OUTCOMES.map(describeRelocation);
    expect(new Set(messages).size).toBe(ALL_OUTCOMES.length);
    messages.forEach((message) => expect(message.length).toBeGreaterThan(0));
  });

  it("tells the user how to set up another device after a move", () => {
    expect(describeRelocation("moved")).toContain("same passphrase");
  });
});

describe("requiresUnlockAgain", () => {
  it("is true only when a different vault is now in use", () => {
    expect(ALL_OUTCOMES.filter(requiresUnlockAgain)).toEqual(["used_existing", "selected"]);
  });
});
