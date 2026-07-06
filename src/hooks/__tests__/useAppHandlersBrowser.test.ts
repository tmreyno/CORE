import { describe, expect, it, vi } from "vitest";
import { useAppHandlers } from "../useAppHandlers";

const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

vi.mock("../project/useProjectDbSync", () => ({
  dbSync: {
    upsertProcessedDatabase: vi.fn(),
  },
}));

function createDeps() {
  return {
    processedDbManager: {
      addDatabases: vi.fn(),
      selectedDatabase: vi.fn(() => null),
      selectDatabase: vi.fn(),
    },
    fileManager: {
      setScanDir: vi.fn(),
    },
    hashManager: {
      hashSelectedFiles: vi.fn(),
      hashAllFiles: vi.fn(),
    },
    projectManager: {
      hasProject: vi.fn(() => true),
    },
    centerPaneTabs: {
      openExportTab: vi.fn(),
      openEvidenceCollection: vi.fn(),
    },
    toast: {
      success: vi.fn(),
      error: vi.fn(),
      info: vi.fn(),
    },
    setLeftPanelTab: vi.fn(),
    setLeftCollapsed: vi.fn(),
    handleScanEvidence: vi.fn(),
    setShowSearchPanel: vi.fn(),
    setShowReportWizard: vi.fn(),
    setShowSettingsPanel: vi.fn(),
    setShowCommandPalette: vi.fn(),
    setShowDedupPanel: vi.fn(),
  };
}

describe("useAppHandlers browser runtime guards", () => {
  it("does not scan processed database locations outside Tauri", async () => {
    const deps = createDeps();
    const handlers = useAppHandlers(deps as any);

    await handlers.handleLocationSelect("/case/processed", "processed");

    expect(mockInvoke).not.toHaveBeenCalled();
    expect(deps.processedDbManager.addDatabases).not.toHaveBeenCalled();
    expect(deps.handleScanEvidence).not.toHaveBeenCalled();
    expect(deps.toast.error).toHaveBeenCalledWith(
      "Scan Unavailable",
      "Processed database scanning is available in the desktop app.",
    );
  });

  it("still routes evidence locations to the evidence scanner", async () => {
    const deps = createDeps();
    const handlers = useAppHandlers(deps as any);

    await handlers.handleLocationSelect("/case/evidence", "evidence");

    expect(deps.fileManager.setScanDir).toHaveBeenCalledWith("/case/evidence");
    expect(deps.handleScanEvidence).toHaveBeenCalledTimes(1);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
