import { describe, expect, it } from "vitest";
import { describeShortcut, shortcutFromKeyPress } from "./shortcut-keys";

const NO_MODIFIERS = { ctrlKey: false, altKey: false, shiftKey: false, metaKey: false };

describe("shortcutFromKeyPress", () => {
  it("joins the held modifiers and the key", () => {
    expect(shortcutFromKeyPress({ ...NO_MODIFIERS, code: "KeyL", ctrlKey: true, shiftKey: true })).toBe("Ctrl+Shift+KeyL");
    expect(shortcutFromKeyPress({ ...NO_MODIFIERS, code: "Digit1", metaKey: true, altKey: true })).toBe("Alt+Cmd+Digit1");
  });

  it("waits while only a modifier is held", () => {
    expect(shortcutFromKeyPress({ ...NO_MODIFIERS, code: "ShiftLeft", shiftKey: true })).toBeNull();
    expect(shortcutFromKeyPress({ ...NO_MODIFIERS, code: "MetaRight", metaKey: true })).toBeNull();
  });

  it("keeps a bare key so the backend can explain why it is refused", () => {
    expect(shortcutFromKeyPress({ ...NO_MODIFIERS, code: "KeyL" })).toBe("KeyL");
  });
});

describe("describeShortcut", () => {
  it("shows letters, digits and the platform-neutral modifier plainly", () => {
    expect(describeShortcut("CmdOrCtrl+Shift+A")).toBe("Cmd/Ctrl + Shift + A");
    expect(describeShortcut("Ctrl+Alt+KeyL")).toBe("Ctrl + Alt + L");
    expect(describeShortcut("Cmd+Digit7")).toBe("Cmd + 7");
    expect(describeShortcut("Ctrl+F5")).toBe("Ctrl + F5");
  });
});
