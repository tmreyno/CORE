import { describe, expect, it, vi } from "vitest";
import { createRoot } from "solid-js";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

import { invoke } from "@tauri-apps/api/core";
import { useActivityTimeline, type FFXProject } from "../useActivityTimeline";

const mockInvoke = vi.mocked(invoke);
const BROWSER_TIMELINE_MESSAGE =
  "Activity timeline analysis is available in the desktop app.";

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
  let hook!: ReturnType<typeof useActivityTimeline>;
  let dispose!: () => void;
  createRoot((d) => {
    dispose = d;
    hook = useActivityTimeline();
  });
  return { hook, dispose };
}

describe("useActivityTimeline browser runtime guards", () => {
  it("does not invoke timeline backend commands outside Tauri", async () => {
    const { hook, dispose } = createHook();

    expect(await hook.computeVisualization(project)).toBeNull();
    expect(await hook.exportTimeline(project, "examiner")).toBeNull();
    expect(await hook.exportTimelineJson(project, "examiner")).toBeNull();

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(hook.visualization()).toBeNull();
    expect(hook.error()).toBe(BROWSER_TIMELINE_MESSAGE);
    expect(hook.loading()).toBe(false);
    dispose();
  });
});
