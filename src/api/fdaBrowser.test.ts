import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../__tests__/setup";
import { checkFullDiskAccess, openFullDiskAccessSettings } from "./fda";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("Full Disk Access API browser runtime guards", () => {
  it("returns a browser-safe status without invoking native commands", async () => {
    await expect(checkFullDiskAccess()).resolves.toEqual({
      hasFullDiskAccess: false,
      message: "Full Disk Access checks are available in the desktop app.",
      blockedPaths: [],
    });

    await openFullDiskAccessSettings();

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
