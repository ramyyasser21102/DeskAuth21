import type { Settings } from "@/lib/api";

/** How long a revealed code stays readable before it hides again. */
export const REVEAL_SECONDS = 10;

export type CaptureWarning = "protection-off" | "unsupported";

/** Whether the system is really being asked to keep the window out of captures. */
export function isCaptureProtectionActive(settings: Settings): boolean {
  return settings.capture_protection && settings.capture_protection_supported;
}

/** Why the person should be told their codes can be captured, or `null` when they are covered. */
export function captureWarningFor(settings: Settings): CaptureWarning | null {
  if (!settings.capture_protection_supported) return "unsupported";
  return settings.capture_protection ? null : "protection-off";
}

/** "123 456" becomes "••• •••": the shape stays, the digits do not. */
export function maskCode(code: string): string {
  return code.replace(/\d/g, "•");
}

export type BannerAction = "turn-on-protection" | "hide-codes";

export interface BannerContent {
  title: string;
  note: string;
  action: BannerAction | null;
}

const OFF_TITLE = "Screenshots and screen sharing can capture your codes.";
const UNSUPPORTED_TITLE = "This system cannot block screen capture, so screenshots and screen sharing can capture your codes.";
const HIDDEN_NOTE = "They stay hidden until you reveal them.";
const VISIBLE_NOTE = "Hide your codes until you reveal them to limit the risk.";

/** What the warning says and which single step would help, or `null` when nothing needs saying. */
export function bannerContentFor(settings: Settings): BannerContent | null {
  const warning = captureWarningFor(settings);
  if (!warning) return null;
  const note = settings.hide_codes ? HIDDEN_NOTE : VISIBLE_NOTE;
  if (warning === "protection-off") return { title: OFF_TITLE, note, action: "turn-on-protection" };
  return { title: UNSUPPORTED_TITLE, note, action: settings.hide_codes ? null : "hide-codes" };
}
