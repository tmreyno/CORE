import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../../__tests__/setup";
import { createProjectIO } from "../useProjectIO";
import type {
  ActivityLogger,
  BuildProjectOptions,
  ProjectStateSetters,
  ProjectStateSignals,
} from "../types";

vi.mock("../../../utils/platform", () => ({
  isTauri: true,
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
    currentUser: () => "Desktop Examiner",
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

describe("Project I/O save serialization", () => {
  it("persists center entry tab metadata used by evidence engines", async () => {
    const { io } = makeProjectIO();
    mockInvoke.mockImplementation((command: string) => {
      if (command === "get_app_version") return Promise.resolve("0.1.112");
      if (command === "project_save") {
        return Promise.resolve({ success: true, path: "/cases/seed.cffx" });
      }
      return Promise.resolve(undefined);
    });

    const options: BuildProjectOptions = {
      rootPath: "/cases/1827-1001/1.Evidence",
      centerTabs: [
        {
          id: "entry:/Users/terry/Documents/report.pdf",
          type: "entry",
          title: "report.pdf",
          subtitle: "export.AD1",
          entry: {
            containerPath: "/cases/1827-1001/1.Evidence/export.AD1",
            entryPath: "/Users/terry/Documents/report.pdf",
            name: "report.pdf",
            size: 4096,
            isDir: false,
            isVfsEntry: false,
            isArchiveEntry: false,
            containerType: "ad1",
            dataAddr: 8192,
            itemAddr: 4096,
            compressedSize: 2048,
            dataEndAddr: 10240,
            metadataAddr: 12288,
            firstChildAddr: null,
          },
        },
      ],
      activeTabId: "entry:/Users/terry/Documents/report.pdf",
      viewMode: "document",
      hashHistory: new Map(),
      processedDatabases: [],
      selectedProcessedDb: null,
    };

    await expect(io.saveProject(options, "/cases/seed.cffx")).resolves.toEqual({
      success: true,
      path: "/cases/seed.cffx",
    });

    expect(mockInvoke).toHaveBeenCalledWith(
      "project_save",
      expect.objectContaining({
        path: "/cases/seed.cffx",
        project: expect.objectContaining({
          tabs: [
            expect.objectContaining({
              entry_size: 4096,
              entry_is_dir: false,
              entry_is_vfs_entry: false,
              entry_is_archive_entry: false,
              entry_container_type: "ad1",
              entry_data_addr: 8192,
              entry_item_addr: 4096,
              entry_compressed_size: 2048,
              entry_data_end_addr: 10240,
              entry_metadata_addr: 12288,
              entry_first_child_addr: null,
            }),
          ],
        }),
      }),
    );
  });
});
