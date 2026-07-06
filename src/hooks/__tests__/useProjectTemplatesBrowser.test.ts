import { describe, expect, it, vi } from "vitest";
import { createRoot } from "solid-js";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

import { invoke } from "@tauri-apps/api/core";
import { useProjectTemplates } from "../useProjectTemplates";

const mockInvoke = vi.mocked(invoke);
const BROWSER_TEMPLATE_MESSAGE =
  "Project template tools are available in the desktop app.";

function createHook() {
  let hook!: ReturnType<typeof useProjectTemplates>;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = useProjectTemplates();
  });
  return { hook, dispose };
}

describe("useProjectTemplates browser runtime guards", () => {
  it("does not invoke template backend commands outside Tauri", async () => {
    const { hook, dispose } = createHook();

    expect(await hook.listTemplates()).toEqual([]);
    expect(await hook.getTemplate("mobile_forensics")).toBeNull();
    expect(await hook.applyTemplate("/case/project.cffx", "mobile_forensics")).toBe(false);
    expect(await hook.createFromProject("/case/project.cffx", "Template", "General")).toBeNull();
    expect(await hook.deleteTemplate("custom-template")).toBe(false);
    expect(await hook.exportTemplate("mobile_forensics", "/tmp/template.json")).toBe(false);
    expect(await hook.importTemplate("/tmp/template.json")).toBeNull();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(hook.error()).toBe(BROWSER_TEMPLATE_MESSAGE);
    expect(hook.loading()).toBe(false);
    expect(hook.templates()).toEqual([]);
    expect(hook.currentTemplate()).toBeNull();
    dispose();
  });
});
