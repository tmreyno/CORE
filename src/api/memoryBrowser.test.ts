import { describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../__tests__/setup";
import {
  cancelMemoryCapture,
  captureMemory,
  getMemoryCaptureInfo,
  listenMemoryCaptureProgress,
} from "./memory";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("memory API browser runtime guards", () => {
  it("returns unsupported memory capture info without native commands", async () => {
    const info = await getMemoryCaptureInfo();

    expect(info).toMatchObject({
      platform: "browser",
      captureSupported: false,
      captureMethod: "unavailable",
    });
    expect(info.unsupportedReason).toContain("desktop app");
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not invoke native cancel or progress listeners in browser preview", async () => {
    const callback = vi.fn();
    const unlisten = await listenMemoryCaptureProgress(callback);

    await expect(cancelMemoryCapture()).resolves.toBeUndefined();
    unlisten();

    expect(callback).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
    expect(mockListen).not.toHaveBeenCalled();
  });

  it("rejects memory capture with a desktop-only error before invoking native code", async () => {
    await expect(captureMemory("/case/memory.mem", true)).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
