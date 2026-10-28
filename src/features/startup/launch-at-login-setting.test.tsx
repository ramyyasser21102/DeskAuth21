import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getLaunchAtLogin, setLaunchAtLogin } from "@/lib/api";
import { LaunchAtLoginSetting } from "./launch-at-login-setting";

vi.mock("@/lib/api", () => ({ getLaunchAtLogin: vi.fn(), setLaunchAtLogin: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function renderSetting(isOn: boolean) {
  vi.mocked(getLaunchAtLogin).mockResolvedValue(isOn);
  render(<LaunchAtLoginSetting />);
  await act(async () => {});
}

const loginSwitch = () => screen.getByRole("switch", { name: "Launch at login" });

describe("LaunchAtLoginSetting", () => {
  it("shows what the system says", async () => {
    await renderSetting(true);
    expect(loginSwitch().getAttribute("aria-checked")).toBe("true");
  });

  it("turns it on and shows the result the system confirmed", async () => {
    await renderSetting(false);
    vi.mocked(setLaunchAtLogin).mockResolvedValue(true);
    await act(async () => { fireEvent.click(loginSwitch()); });
    expect(setLaunchAtLogin).toHaveBeenCalledWith(true);
    expect(loginSwitch().getAttribute("aria-checked")).toBe("true");
  });

  it("says why it failed and goes back to what the system really has", async () => {
    await renderSetting(false);
    vi.mocked(setLaunchAtLogin).mockRejectedValue("could not change the launch-at-login setting: denied");
    await act(async () => { fireEvent.click(loginSwitch()); });
    expect(screen.getByRole("alert").textContent).toContain("denied");
    expect(loginSwitch().getAttribute("aria-checked")).toBe("false");
  });
});
