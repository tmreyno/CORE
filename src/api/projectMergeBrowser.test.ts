import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../__tests__/setup";
import { analyzeProjects, executeMerge } from "./projectMerge";

vi.mock("../utils/platform", () => ({
  isTauri: false,
}));

describe("project merge API browser runtime guards", () => {
  it("rejects merge analysis before native commands", async () => {
    await expect(analyzeProjects(["/case/a.cffx"])).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("returns a failed merge result without native commands", async () => {
    await expect(
      executeMerge(["/case/a.cffx"], "/case/merged.cffx", "Merged Case"),
    ).resolves.toMatchObject({
      success: false,
      cffxPath: "/case/merged.cffx",
      error: expect.stringContaining("desktop app"),
    });

    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
