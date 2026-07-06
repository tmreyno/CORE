import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { createProjectIO } from "../useProjectIO";
import type {
  ActivityLogger,
  ProjectStateSetters,
  ProjectStateSignals,
} from "../types";

vi.mock("../../../utils/platform", () => ({
  isTauri: false,
}));

vi.mock("../../../components/preferences", () => ({
  addRecentProject: vi.fn(),
}));

function makeProjectIO() {
  const signals: ProjectStateSignals = {
    project: () => null,
    projectPath: () => null,
    modified: () => false,
    error: () => null,
    loading: () => false,
    currentUser: () => "Browser Examiner",
    currentSessionId: () => null,
    autoSaveEnabled: () => false,
    lastAutoSave: () => null,
  };

  const setters: ProjectStateSetters = {
    setProject: vi.fn(),
    setProjectPath: vi.fn(),
    setModified: vi.fn(),
    setError: vi.fn(),
    setLoading: vi.fn(),
    setCurrentUser: vi.fn(),
    setCurrentSessionId: vi.fn(),
    setAutoSaveEnabled: vi.fn(),
    setLastAutoSave: vi.fn(),
  };

  const logger: ActivityLogger = {
    logActivity: vi.fn(),
    flushActivity: vi.fn(),
  };

  const autoSave = {
    startAutoSave: vi.fn(),
    stopAutoSave: vi.fn(),
  };

  return {
    io: createProjectIO(signals, setters, vi.fn(), logger, autoSave),
    setters,
  };
}

describe("Project I/O browser runtime guards", () => {
  it("uses browser-safe project path helpers without native commands", async () => {
    const { io } = makeProjectIO();

    await expect(io.checkProjectExists("/Cases/1827-1001")).resolves.toBeNull();
    await expect(io.getDefaultProjectPath("/Cases/1827-1001/")).resolves.toBe("1827-1001.cffx");

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("clears project state without native DB close/checkpoint/menu commands", async () => {
    const { io, setters } = makeProjectIO();
    const progress = vi.fn();

    await expect(io.clearProject({ onProgress: progress })).resolves.toEqual({
      success: true,
      flushTimedOut: false,
    });

    expect(setters.setProject).toHaveBeenCalledWith(null);
    expect(setters.setProjectPath).toHaveBeenCalledWith(null);
    expect(setters.setCurrentSessionId).toHaveBeenCalledWith(null);
    expect(progress).toHaveBeenCalledWith({
      step: "checkpoint-db",
      status: "completed",
      detail: "Project database checkpoint skipped in browser preview.",
    });
    expect(progress).toHaveBeenCalledWith({
      step: "close-db",
      status: "completed",
      detail: "Project database close skipped in browser preview.",
    });
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
