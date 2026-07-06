import { createRoot } from "solid-js";
import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { loadExaminerProfile, useExaminerProfile } from "../useExaminerProfile";

vi.mock("../../../utils/platform", () => ({
  isTauri: false,
}));

describe("useExaminerProfile browser runtime guard", () => {
  it("does not invoke project DB ui-state commands", async () => {
    const profile = await loadExaminerProfile();
    expect(profile).toEqual({
      name: "",
      title: "",
      organization: "",
      badge_number: "",
      email: "",
      phone: "",
    });

    let hook!: ReturnType<typeof useExaminerProfile>;
    let dispose!: () => void;
    createRoot((d) => {
      dispose = d;
      hook = useExaminerProfile();
    });

    await hook.refresh();

    expect(hook.profile()).toEqual(profile);
    expect(mockInvoke).not.toHaveBeenCalled();
    dispose();
  });
});
