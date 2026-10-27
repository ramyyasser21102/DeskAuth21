import { describe, expect, it } from "vitest";
import { DEFAULT_SETTINGS } from "@/test-support/settings-fixture";
import { bannerContentFor, captureWarningFor, isCaptureProtectionActive, maskCode } from "./privacy-state";

describe("captureWarningFor", () => {
  it("is quiet while protection is on and supported", () => {
    expect(captureWarningFor(DEFAULT_SETTINGS)).toBeNull();
    expect(isCaptureProtectionActive(DEFAULT_SETTINGS)).toBe(true);
  });

  it("warns when the person turned protection off", () => {
    const off = { ...DEFAULT_SETTINGS, capture_protection: false };
    expect(captureWarningFor(off)).toBe("protection-off");
    expect(isCaptureProtectionActive(off)).toBe(false);
  });

  it("warns that the system cannot block capture, whatever the setting says", () => {
    const unsupported = { ...DEFAULT_SETTINGS, capture_protection_supported: false };
    expect(captureWarningFor(unsupported)).toBe("unsupported");
    expect(isCaptureProtectionActive(unsupported)).toBe(false);
  });
});

describe("maskCode", () => {
  it("keeps the grouping and hides every digit", () => {
    expect(maskCode("123 456")).toBe("••• •••");
    expect(maskCode("1234 5678")).toBe("•••• ••••");
  });
});

describe("bannerContentFor", () => {
  it("says nothing while protection is active", () => {
    expect(bannerContentFor(DEFAULT_SETTINGS)).toBeNull();
  });

  it("offers to turn protection back on when the person turned it off", () => {
    const content = bannerContentFor({ ...DEFAULT_SETTINGS, capture_protection: false });
    expect(content?.action).toBe("turn-on-protection");
    expect(content?.title).toContain("can capture your codes");
  });

  it("offers hidden codes when the system cannot block capture and codes are visible", () => {
    const settings = { ...DEFAULT_SETTINGS, capture_protection_supported: false, hide_codes: false };
    expect(bannerContentFor(settings)?.action).toBe("hide-codes");
  });

  it("has no action when the system cannot block capture but codes are already hidden", () => {
    const content = bannerContentFor({ ...DEFAULT_SETTINGS, capture_protection_supported: false });
    expect(content?.action).toBeNull();
    expect(content?.note).toContain("hidden");
  });
});
