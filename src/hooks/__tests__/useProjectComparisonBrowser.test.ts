import { describe, expect, it, vi } from "vitest";
import { createRoot } from "solid-js";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

import { invoke } from "@tauri-apps/api/core";
import { useProjectComparison } from "../useProjectComparison";
import type { FFXProject } from "../useActivityTimeline";

const mockInvoke = vi.mocked(invoke);
const BROWSER_COMPARISON_MESSAGE =
  "Project comparison tools are available in the desktop app.";

const project: FFXProject = {
  name: "Case 1827",
  path: "/case/project.cffx",
  created_at: "2026-07-04T00:00:00Z",
  modified_at: "2026-07-04T00:00:00Z",
  version: "0.1.112",
  bookmarks: [],
  notes: [],
  activity_log: [],
  evidence_items: [],
  metadata: {},
};

function createHook() {
  let hook!: ReturnType<typeof useProjectComparison>;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = useProjectComparison();
  });
  return { hook, dispose };
}

describe("useProjectComparison browser runtime guards", () => {
  it("does not invoke project comparison backend commands outside Tauri", async () => {
    const { hook, dispose } = createHook();

    expect(await hook.compareProjects(project, project)).toBeNull();
    expect(await hook.mergeProjects(project, project)).toBeNull();
    expect(await hook.syncBookmarks(project, project)).toBeNull();
    expect(await hook.syncNotes(project, project)).toBeNull();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(hook.comparison()).toBeNull();
    expect(hook.mergeResult()).toBeNull();
    expect(hook.error()).toBe(BROWSER_COMPARISON_MESSAGE);
    expect(hook.loading()).toBe(false);
    dispose();
  });
});
