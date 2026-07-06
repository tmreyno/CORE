import { describe, expect, it, vi } from "vitest";
import { mockInvoke, mockListen } from "../__tests__/setup";
import {
  getTriageProfiles,
  listenTriageProgress,
  triageCancel,
  triageCollect,
} from "./triage";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("triage API browser runtime guards", () => {
  it("returns empty triage profiles without native commands", async () => {
    await expect(getTriageProfiles()).resolves.toEqual([[], []]);

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not invoke native cancel or progress listeners in browser preview", async () => {
    const callback = vi.fn();
    const unlisten = await listenTriageProgress(callback);

    await expect(triageCancel()).resolves.toBeUndefined();
    unlisten();

    expect(callback).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
    expect(mockListen).not.toHaveBeenCalled();
  });

  it("rejects triage collection with a desktop-only error before invoking native code", async () => {
    await expect(
      triageCollect({
        outputDir: "/case/triage",
        categories: ["browser"],
        scanForSecrets: true,
      }),
    ).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
