import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../__tests__/setup";
import {
  checkPathWritable,
  listDrives,
  remountReadOnly,
  restoreMount,
} from "./drives";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("drives API browser runtime guards", () => {
  it("returns safe defaults without invoking native drive commands", async () => {
    await expect(listDrives()).resolves.toEqual([]);
    await expect(checkPathWritable("/case")).resolves.toMatchObject({
      writable: false,
    });
    await expect(remountReadOnly("/Volumes/source")).resolves.toMatchObject({
      success: false,
      mountPoint: "/Volumes/source",
    });
    await expect(restoreMount("/Volumes/source")).resolves.toMatchObject({
      success: false,
      mountPoint: "/Volumes/source",
    });

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
