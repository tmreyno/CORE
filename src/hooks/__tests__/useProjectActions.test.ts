// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { useProjectActions, type UseProjectActionsDeps, type ProjectActions } from "../useProjectActions";

/**
 * Tests for the useProjectActions hook.
 *
 * The hook bundles project save / load / directory actions.
 * We focus on the save handlers which contain branching logic
 * (success, error, cancelled, no evidence).
 */

// ---------------------------------------------------------------------------
// Mocks — we mock the heavy helpers to test only the hook's orchestration
// ---------------------------------------------------------------------------

vi.mock("../project/projectHelpers", () => ({
  buildSaveOptions: vi.fn(),
  handleLoadProject: vi.fn(),
  handleOpenDirectory: vi.fn(),
  handleProjectSetupComplete: vi.fn(),
}));

import {
  buildSaveOptions as mockBuildSaveOptions,
  handleLoadProject as mockLoadProject,
  handleOpenDirectory as mockOpenDirectory,
  handleProjectSetupComplete as mockProjectSetup,
} from "../project/projectHelpers";

// ---------------------------------------------------------------------------
// Helper factories
// ---------------------------------------------------------------------------

function createMockDeps(overrides: Partial<UseProjectActionsDeps> = {}): UseProjectActionsDeps {
  const noop = () => {};
  const accessor = <T>(val: T) => () => val;

  return {
    fileManager: {
      scanDir: accessor("/evidence"),
      activeFile: accessor(null),
      discoveredFiles: accessor([]),
      fileInfoMap: accessor(new Map()),
      fileStatusMap: accessor(new Map()),
      busy: accessor(false),
      statusKind: accessor("idle"),
      statusMessage: accessor("Ready"),
      selectedFiles: accessor(new Set()),
      selectedCount: accessor(0),
      totalSize: accessor(0),
      recursiveScan: accessor(true),
      typeFilter: accessor(null),
      containerStats: accessor(new Map()),
      allFilesSelected: accessor(false),
      tree: accessor(null),
      filteredTree: accessor(null),
      treeFilter: accessor(""),
      loadProgress: accessor({ show: false, title: "", message: "", current: 0, total: 0 }),
      systemStats: accessor(null),
      setActiveFile: noop,
      setScanDir: noop,
      setRecursiveScan: noop,
      setTypeFilter: noop,
      setTreeFilter: noop,
      setOk: noop,
      scanForFiles: vi.fn(),
      loadFileInfo: vi.fn(),
      loadAllInfo: vi.fn(),
      cancelLoading: noop,
      setupSystemStatsListener: vi.fn(() => Promise.resolve(noop)),
      toggleFileSelection: noop,
      toggleSelectAll: noop,
      hashSingleFile: vi.fn(),
      toggleTypeFilter: noop,
    } as any,
    hashManager: {
      fileHashMap: accessor(new Map()),
      hashHistory: accessor([]),
      selectedHashAlgorithm: accessor("SHA-256"),
      formatHashDate: (d: string) => d,
      getAllStoredHashesSorted: vi.fn(() => []),
      setSelectedHashAlgorithm: noop,
      hashSelectedFiles: vi.fn(),
      hashSingleFile: vi.fn(),
      hashAllFiles: vi.fn(),
      addTransferHashesToHistory: vi.fn(),
    } as any,
    projectManager: {
      projectPath: vi.fn(() => null as string | null),
      projectName: accessor(null),
      projectLocations: accessor(null),
      modified: accessor(false),
      autoSaveEnabled: accessor(false),
      lastAutoSave: accessor(null),
      rootPath: accessor(null),
      bookmarkCount: accessor(0),
      saveProject: vi.fn(),
      loadProject: vi.fn(),
      setAutoSaveCallback: noop,
      setAutoSaveEnabled: noop,
      startAutoSave: noop,
      stopAutoSave: noop,
      logActivity: noop,
    } as any,
    processedDbManager: {
      databases: accessor([]),
      selectedDatabase: accessor(null),
      selectedCaseInfo: accessor(null),
      selectedCategories: accessor([]),
      isSelectedLoading: accessor(false),
      detailView: accessor("summary"),
      setDetailView: noop,
      loadDatabase: vi.fn(),
      setSelectedDatabase: noop,
    } as any,
    centerPaneTabs: {
      tabs: accessor([]),
      activeTabId: accessor(null),
      activeTab: accessor(null),
      activeTabType: accessor(null),
      viewMode: accessor("content"),
      setTabs: vi.fn(),
      setActiveTabId: vi.fn(),
      setViewMode: vi.fn(),
      openEvidenceFile: vi.fn(),
      openContainerEntry: vi.fn(),
      openCaseDocument: vi.fn(),
      openProcessedDatabase: vi.fn(),
      openExportTab: vi.fn(),
      closeTab: vi.fn(),
    } as any,
    toast: {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    },
    openTabs: accessor([]),
    selectedContainerEntry: accessor(null),
    leftWidth: accessor(320),
    rightWidth: accessor(320),
    leftCollapsed: accessor(false),
    rightCollapsed: accessor(false),
    leftPanelTab: accessor("evidence"),
    currentViewMode: accessor("info"),
    entryContentViewMode: accessor("auto"),
    caseDocumentsPath: accessor(null),
    treeExpansionState: accessor(null),
    caseDocuments: accessor(null),
    setLeftWidth: vi.fn(),
    setRightWidth: vi.fn(),
    setLeftCollapsed: vi.fn(),
    setRightCollapsed: vi.fn(),
    setLeftPanelTab: vi.fn(),
    setCurrentViewMode: vi.fn(),
    setEntryContentViewMode: vi.fn(),
    setCaseDocumentsPath: vi.fn(),
    setTreeExpansionState: vi.fn(),
    setSelectedContainerEntry: vi.fn(),
    setOpenTabs: vi.fn(),
    setCaseDocuments: vi.fn(),
    setPendingProjectRoot: vi.fn(),
    setShowProjectWizard: vi.fn(),
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("useProjectActions", () => {
  let deps: UseProjectActionsDeps;
  let actions: ProjectActions;

  beforeEach(() => {
    vi.clearAllMocks();
    deps = createMockDeps();
    actions = useProjectActions(deps);
  });

  // =========================================================================
  // Return shape
  // =========================================================================

  it("returns all expected action functions", () => {
    expect(actions.getSaveOptions).toBeTypeOf("function");
    expect(actions.handleSaveProject).toBeTypeOf("function");
    expect(actions.handleSaveProjectAs).toBeTypeOf("function");
    expect(actions.handleLoadProject).toBeTypeOf("function");
    expect(actions.handleOpenDirectory).toBeTypeOf("function");
    expect(actions.handleProjectSetupComplete).toBeTypeOf("function");
  });

  // =========================================================================
  // getSaveOptions
  // =========================================================================

  describe("getSaveOptions", () => {
    it("delegates to buildSaveOptions with correct params", () => {
      actions.getSaveOptions();
      expect(mockBuildSaveOptions).toHaveBeenCalledOnce();
    });

    it("returns null when buildSaveOptions returns null", () => {
      vi.mocked(mockBuildSaveOptions).mockReturnValue(null);
      expect(actions.getSaveOptions()).toBeNull();
    });

    it("returns options when buildSaveOptions returns an object", () => {
      const options = { rootPath: "/evidence", tabs: [] };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      expect(actions.getSaveOptions()).toBe(options);
    });
  });

  // =========================================================================
  // handleSaveProject
  // =========================================================================

  describe("handleSaveProject", () => {
    it("shows error toast when no evidence directory is open", async () => {
      vi.mocked(mockBuildSaveOptions).mockReturnValue(null);

      await actions.handleSaveProject();

      expect(deps.toast.error).toHaveBeenCalledWith("No Evidence", "Open an evidence directory first");
      expect(deps.projectManager.saveProject).not.toHaveBeenCalled();
    });

    it("saves to existing path when projectPath() is set", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      vi.mocked(deps.projectManager.projectPath).mockReturnValue("/projects/test.cffx");
      vi.mocked(deps.projectManager.saveProject).mockResolvedValue({ success: true });

      await actions.handleSaveProject();

      expect(deps.projectManager.saveProject).toHaveBeenCalledWith(options, "/projects/test.cffx");
      expect(deps.toast.success).toHaveBeenCalledWith("Project Saved", "Your project has been saved");
    });

    it("shows save dialog when no existing projectPath", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      vi.mocked(deps.projectManager.projectPath).mockReturnValue(null);
      vi.mocked(deps.projectManager.saveProject).mockResolvedValue({ success: true });

      await actions.handleSaveProject();

      expect(deps.projectManager.saveProject).toHaveBeenCalledWith(options);
      expect(deps.toast.success).toHaveBeenCalledWith("Project Saved", "Your project has been saved");
    });

    it("shows error toast on save failure", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      vi.mocked(deps.projectManager.projectPath).mockReturnValue(null);
      vi.mocked(deps.projectManager.saveProject).mockResolvedValue({
        success: false,
        error: "Disk full",
      });

      await actions.handleSaveProject();

      expect(deps.toast.error).toHaveBeenCalledWith("Save Failed", "Disk full");
    });

    it("does NOT toast when user cancels save dialog", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      vi.mocked(deps.projectManager.projectPath).mockReturnValue(null);
      vi.mocked(deps.projectManager.saveProject).mockResolvedValue({
        success: false,
        error: "Save cancelled",
      });

      await actions.handleSaveProject();

      expect(deps.toast.error).not.toHaveBeenCalled();
      expect(deps.toast.success).not.toHaveBeenCalled();
    });

    it("shows error toast when saveProject throws", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      vi.mocked(deps.projectManager.projectPath).mockReturnValue(null);
      vi.mocked(deps.projectManager.saveProject).mockRejectedValue(
        new Error("Permission denied"),
      );

      await actions.handleSaveProject();

      expect(deps.toast.error).toHaveBeenCalledWith("Save Failed", "Permission denied");
    });
  });

  // =========================================================================
  // handleSaveProjectAs
  // =========================================================================

  describe("handleSaveProjectAs", () => {
    it("shows error toast when no evidence directory is open", async () => {
      vi.mocked(mockBuildSaveOptions).mockReturnValue(null);

      await actions.handleSaveProjectAs();

      expect(deps.toast.error).toHaveBeenCalledWith("No Evidence", "Open an evidence directory first");
    });

    it("always shows save dialog (never uses existing path)", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      // Even though projectPath is set, Save As should NOT use it
      vi.mocked(deps.projectManager.projectPath).mockReturnValue("/projects/test.cffx");
      vi.mocked(deps.projectManager.saveProject).mockResolvedValue({ success: true });

      await actions.handleSaveProjectAs();

      // Only one argument (no path), forcing the dialog
      expect(deps.projectManager.saveProject).toHaveBeenCalledWith(options);
      expect(deps.toast.success).toHaveBeenCalledWith(
        "Project Saved",
        "Your project has been saved to a new location",
      );
    });

    it("shows error toast on save failure", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      vi.mocked(deps.projectManager.saveProject).mockResolvedValue({
        success: false,
        error: "Write error",
      });

      await actions.handleSaveProjectAs();

      expect(deps.toast.error).toHaveBeenCalledWith("Save Failed", "Write error");
    });

    it("does NOT toast when user cancels save dialog", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      vi.mocked(deps.projectManager.saveProject).mockResolvedValue({
        success: false,
        error: "Save cancelled",
      });

      await actions.handleSaveProjectAs();

      expect(deps.toast.error).not.toHaveBeenCalled();
    });

    it("catches and toasts thrown exceptions", async () => {
      const options = { rootPath: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue(options as any);
      vi.mocked(deps.projectManager.saveProject).mockRejectedValue("Unknown error");

      await actions.handleSaveProjectAs();

      expect(deps.toast.error).toHaveBeenCalledWith("Save Failed", "Unknown error");
    });
  });

  // =========================================================================
  // handleLoadProject
  // =========================================================================

  describe("handleLoadProject", () => {
    it("delegates to loadProjectHandler", async () => {
      await actions.handleLoadProject();

      expect(mockLoadProject).toHaveBeenCalledOnce();
    });

    it("passes projectPath when provided", async () => {
      await actions.handleLoadProject("/projects/case.cffx");

      const callArg = vi.mocked(mockLoadProject).mock.calls[0][0];
      expect(callArg).toHaveProperty("projectPath", "/projects/case.cffx");
    });

    it("passes undefined projectPath when not provided", async () => {
      await actions.handleLoadProject();

      const callArg = vi.mocked(mockLoadProject).mock.calls[0][0];
      expect(callArg).toHaveProperty("projectPath", undefined);
    });

    it("passes toast to loadProjectHandler", async () => {
      await actions.handleLoadProject();

      const callArg = vi.mocked(mockLoadProject).mock.calls[0][0];
      expect(callArg.toast).toBe(deps.toast);
    });
  });

  // =========================================================================
  // handleOpenDirectory
  // =========================================================================

  describe("handleOpenDirectory", () => {
    it("delegates to openDirectoryHandler", () => {
      actions.handleOpenDirectory();

      expect(mockOpenDirectory).toHaveBeenCalledOnce();
    });

    it("passes setPendingProjectRoot and setShowProjectWizard", () => {
      actions.handleOpenDirectory();

      const callArg = vi.mocked(mockOpenDirectory).mock.calls[0][0];
      expect(callArg.setPendingProjectRoot).toBe(deps.setPendingProjectRoot);
      expect(callArg.setShowProjectWizard).toBe(deps.setShowProjectWizard);
    });
  });

  // =========================================================================
  // handleProjectSetupComplete
  // =========================================================================

  describe("handleProjectSetupComplete", () => {
    it("delegates to projectSetupHandler with locations", async () => {
      const locations = {
        evidence_path: "/evidence",
        processed_db_path: "/processed",
        case_documents_path: "/docs",
      };

      await actions.handleProjectSetupComplete(locations as any);

      expect(mockProjectSetup).toHaveBeenCalledOnce();
      expect(vi.mocked(mockProjectSetup).mock.calls[0][1]).toBe(locations);
    });

    it("passes a working getSaveOptions to the helper", async () => {
      const locations = { evidence_path: "/evidence" };
      vi.mocked(mockBuildSaveOptions).mockReturnValue({ rootPath: "/evidence" } as any);

      await actions.handleProjectSetupComplete(locations as any);

      const helperDeps = vi.mocked(mockProjectSetup).mock.calls[0][0];
      const result = helperDeps.getSaveOptions?.();
      expect(result).toEqual({ rootPath: "/evidence" });
    });
  });
});
