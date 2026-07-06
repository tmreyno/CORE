import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

import { useExportState } from "../useExportState";

function createToast() {
  return {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  };
}

describe("useExportState browser runtime guards", () => {
  it("initializes export filenames from provided stats without native system-info calls", () => {
    let state!: ReturnType<typeof useExportState>;
    let dispose!: () => void;

    createRoot((d) => {
      dispose = d;
      state = useExportState({
        toast: createToast(),
        projectName: "Browser Case",
        systemStats: {
          hostname: "lab-host",
          systemSerialNumber: "SERIAL-12345",
        } as any,
      });
    });

    expect(mockInvoke).not.toHaveBeenCalledWith("get_hostname");
    expect(mockInvoke).not.toHaveBeenCalledWith("get_current_username");
    expect(mockInvoke).not.toHaveBeenCalledWith("get_system_stats");
    expect(state.ewfImageName()).toContain("Browser_Case-12345-lab-host-user-");
    expect(state.l01ImageName()).toBe(state.ewfImageName());
    expect(state.rawImageName()).toBe(state.ewfImageName());
    expect(state.aff4ImageName()).toBe(state.ewfImageName());
    dispose();
  });

  it("uses browser-safe fallback filename segments when stats are unavailable", () => {
    let state!: ReturnType<typeof useExportState>;
    let dispose!: () => void;

    createRoot((d) => {
      dispose = d;
      state = useExportState({
        toast: createToast(),
        projectName: "Browser Case",
      });
    });

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(state.ewfImageName()).toContain("Browser_Case-NOSN0-host-user-");
    dispose();
  });
});
