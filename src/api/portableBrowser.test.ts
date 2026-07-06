import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../__tests__/setup";
import { ensurePortableDirs, getPortableStatus } from "./portable";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("portable API browser runtime guards", () => {
  it("returns inactive portable status without native commands", async () => {
    await expect(getPortableStatus()).resolves.toEqual({
      isPortable: false,
      config: null,
    });

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("rejects portable directory setup before invoking native code", async () => {
    await expect(ensurePortableDirs()).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
