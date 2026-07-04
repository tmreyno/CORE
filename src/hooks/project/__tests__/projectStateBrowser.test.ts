import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { createProjectState, getAppVersion, getCurrentUsername } from "../useProjectState";

vi.mock("../../../utils/platform", () => ({
  isTauri: false,
}));

describe("project state browser runtime guards", () => {
  it("does not invoke native username or version commands", async () => {
    expect(await getCurrentUsername()).toBe("unknown");
    expect(await getAppVersion()).toBeTypeOf("string");

    const { signals } = createProjectState();
    await Promise.resolve();

    expect(signals.currentUser()).toBe("unknown");
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
