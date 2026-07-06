import { describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../__tests__/setup";
import {
  checkPrivilege,
  getDeviceSize,
  listPhysicalDisks,
  listenDeviceReadProgress,
  readRawDevice,
  requestElevation,
} from "./device";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("device API browser runtime guards", () => {
  it("returns safe privilege and disk defaults without native commands", async () => {
    await expect(checkPrivilege()).resolves.toMatchObject({
      isElevated: false,
      elevationRequired: true,
    });
    await expect(listPhysicalDisks()).resolves.toEqual([]);
    await expect(requestElevation()).resolves.toContain("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("rejects raw device operations before invoking native code", async () => {
    await expect(getDeviceSize("/dev/rdisk2")).rejects.toThrow("desktop app");
    await expect(readRawDevice("/dev/rdisk2", "/case/disk.raw")).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not register native progress listeners in browser preview", async () => {
    const callback = vi.fn();
    const unlisten = await listenDeviceReadProgress(callback);

    unlisten();

    expect(callback).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
    expect(mockListen).not.toHaveBeenCalled();
  });
});
