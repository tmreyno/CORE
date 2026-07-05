// =============================================================================
// projectSetup — createDocumentEntry tests
// =============================================================================

import { describe, it, expect, vi } from "vitest";
import { createDocumentEntry, handleOpenDirectory, handleProjectSetupComplete } from "../projectSetup";
import { open as mockOpen } from "@tauri-apps/plugin-dialog";
import type { CaseDocument } from "../../../types";
import { mockInvoke } from "../../../__tests__/setup";

function makeDoc(overrides: Partial<CaseDocument> = {}): CaseDocument {
  return {
    path: "/evidence/report.pdf",
    filename: "report.pdf",
    size: 12345,
    format: "pdf",
    modified: "2026-01-15",
    document_type: "forensic_report",
    case_number: "C-001",
    evidence_id: "EV-01",
    ...overrides,
  } as CaseDocument;
}

describe("createDocumentEntry", () => {
  it("maps CaseDocument to SelectedEntry", () => {
    const entry = createDocumentEntry(makeDoc());
    expect(entry.containerPath).toBe("/evidence/report.pdf");
    expect(entry.entryPath).toBe("/evidence/report.pdf");
    expect(entry.name).toBe("report.pdf");
    expect(entry.size).toBe(12345);
    expect(entry.isDir).toBe(false);
  });

  it("defaults isDiskFile to true", () => {
    const entry = createDocumentEntry(makeDoc());
    expect(entry.isDiskFile).toBe(true);
  });

  it("accepts isDiskFile = false", () => {
    const entry = createDocumentEntry(makeDoc(), false);
    expect(entry.isDiskFile).toBe(false);
  });

  it("sets containerType from format", () => {
    const entry = createDocumentEntry(makeDoc({ format: "docx" }));
    expect(entry.containerType).toBe("docx");
  });

  it("falls back to 'file' when format is undefined", () => {
    const entry = createDocumentEntry(makeDoc({ format: undefined }));
    expect(entry.containerType).toBe("file");
  });

  it("populates metadata from document fields", () => {
    const entry = createDocumentEntry(makeDoc());
    expect(entry.metadata?.document_type).toBe("forensic_report");
    expect(entry.metadata?.case_number).toBe("C-001");
    expect(entry.metadata?.evidence_id).toBe("EV-01");
    expect(entry.metadata?.format).toBe("pdf");
    expect(entry.metadata?.modified).toBe("2026-01-15");
  });

  it("handles missing optional metadata fields", () => {
    const entry = createDocumentEntry(
      makeDoc({ document_type: undefined, case_number: undefined, evidence_id: undefined }),
    );
    expect(entry.metadata?.document_type).toBeUndefined();
    expect(entry.metadata?.case_number).toBeUndefined();
    expect(entry.metadata?.evidence_id).toBeUndefined();
  });
});

describe("handleOpenDirectory", () => {
  it("opens the setup wizard without a native directory picker outside Tauri", async () => {
    const params = {
      setPendingProjectRoot: vi.fn(),
      setShowProjectWizard: vi.fn(),
      toast: {
        success: vi.fn(),
        error: vi.fn(),
        info: vi.fn(),
      },
    };

    await handleOpenDirectory(params);

    expect(mockOpen).not.toHaveBeenCalled();
    expect(params.setPendingProjectRoot).toHaveBeenCalledWith(null);
    expect(params.setShowProjectWizard).toHaveBeenCalledWith(true);
    expect(params.toast.info).toHaveBeenCalledWith(
      "Browser Preview",
      expect.stringContaining("New Project opens in preview mode"),
    );
  });
});

describe("handleProjectSetupComplete browser runtime guards", () => {
  it("initializes browser project state without desktop filesystem or database work", async () => {
    const fileManager = {
      setScanDir: vi.fn(),
      scanForFiles: vi.fn(),
      discoveredFiles: vi.fn(() => []),
      fileInfoMap: vi.fn(() => new Map()),
      loadStoredHashesInBackground: vi.fn(),
    };
    const hashManager = {
      importPreloadedStoredHashes: vi.fn(),
      hashHistory: vi.fn(() => []),
    };
    const processedDbManager = {
      databases: vi.fn(() => []),
      selectedDatabase: vi.fn(() => null),
      addDatabases: vi.fn(),
      selectDatabase: vi.fn(),
    };
    const projectManager = {
      createProject: vi.fn().mockResolvedValue({}),
      updateLocations: vi.fn(),
      saveProject: vi.fn(),
      logActivity: vi.fn(),
    };
    const params = {
      fileManager,
      hashManager,
      processedDbManager,
      projectManager,
      setShowProjectWizard: vi.fn(),
      setCaseDocumentsPath: vi.fn(),
      setLeftPanelTab: vi.fn(),
      setPendingProjectRoot: vi.fn(),
      toast: {
        success: vi.fn(),
        error: vi.fn(),
        info: vi.fn(),
      },
    };

    await handleProjectSetupComplete(params as any, {
      projectName: "Browser Case",
      projectRoot: "/browser/project",
      evidencePath: "/browser/project/Evidence",
      processedDbPath: "/browser/project/Processed",
      caseDocumentsPath: "/browser/project/Case.Documents",
      exportsPath: "/browser/project/Exports",
      discoveredEvidence: [],
      discoveredDatabases: [],
      loadStoredHashes: true,
    });

    expect(projectManager.createProject).toHaveBeenCalledWith(
      "/browser/project",
      "Browser Case",
      undefined,
      undefined,
      undefined,
    );
    expect(projectManager.updateLocations).toHaveBeenCalledWith(
      expect.objectContaining({
        project_root: "/browser/project",
        evidence_path: "/browser/project/Evidence",
        case_documents_path: "/browser/project/Case.Documents",
      }),
    );
    expect(fileManager.scanForFiles).not.toHaveBeenCalled();
    expect(fileManager.loadStoredHashesInBackground).not.toHaveBeenCalled();
    expect(processedDbManager.addDatabases).not.toHaveBeenCalled();
    expect(projectManager.saveProject).not.toHaveBeenCalled();
    expect(mockInvoke).not.toHaveBeenCalled();
    expect(params.toast.info).toHaveBeenCalledWith(
      "Browser Preview",
      expect.stringContaining("Folder creation"),
    );
  });
});
