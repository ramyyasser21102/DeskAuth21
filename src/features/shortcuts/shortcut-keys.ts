export interface KeyPress {
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

const MODIFIER_CODE_PREFIXES = ["Control", "Alt", "Shift", "Meta", "OS"];

const isModifierKey = (code: string) => MODIFIER_CODE_PREFIXES.some((prefix) => code.startsWith(prefix));

/** The text the app stores for a key press, or `null` while only modifiers are held. */
export function shortcutFromKeyPress(press: KeyPress): string | null {
  if (isModifierKey(press.code)) return null;
  const modifiers = [
    press.ctrlKey && "Ctrl",
    press.altKey && "Alt",
    press.shiftKey && "Shift",
    press.metaKey && "Cmd",
  ].filter((name): name is string => name !== false);
  return [...modifiers, press.code].join("+");
}

const READABLE_PARTS: Record<string, string> = { CmdOrCtrl: "Cmd/Ctrl", Cmd: "Cmd", Super: "Cmd" };

const KEY_CODE_PATTERN = /^(?:Key|Digit)(.)$/;

function describePart(part: string): string {
  return READABLE_PARTS[part] ?? KEY_CODE_PATTERN.exec(part)?.[1] ?? part;
}

/** "CmdOrCtrl+Shift+KeyA" becomes "Cmd/Ctrl + Shift + A". */
export function describeShortcut(shortcut: string): string {
  return shortcut.split("+").map(describePart).join(" + ");
}
