import { describe, expect, it, vi } from "vitest";
import { createRoot } from "solid-js";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

import { invoke } from "@tauri-apps/api/core";
import { useProjectRecovery } from "../useProjectRecovery";

const mockInvoke = vi.mocked(invoke);
const BROWSER_RECOVERY_MESSAGE =
  "Project recovery tools are available in the desktop app.";

function createHook() {
  let hook!: ReturnType<typeof useProjectRecovery>;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = useProjectRecovery();
  });
  return { hook, dispose };
}

describe("useProjectRecovery browser runtime guards", () => {
  it("does not invoke project recovery backend commands outside Tauri", async () => {
    const { hook, dispose } = createHook();

    expect(await hook.createBackup("/case/project.cffx")).toBeNull();
    expect(await hook.createVersionBackup("/case/project.cffx")).toBeNull();
    expect(await hook.listVersions("/case/project.cffx")).toEqual([]);
    expect(await hook.checkRecovery("/case/project.cffx")).toBeNull();
    expect(await hook.recoverFromAutosave("/case/project.cffx")).toBeNull();
    expect(await hook.clearAutosave("/case/project.cffx")).toBe(false);
    expect(await hook.checkHealth("/case/project.cffx")).toBeNull();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(hook.error()).toBe(BROWSER_RECOVERY_MESSAGE);
    expect(hook.loading()).toBe(false);
    dispose();
  });
});
