// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi } from "vitest";
import { handleLoadProject, restoreCenterTabs } from "../projectLoader";
import type { CaseDocument, DiscoveredFile } from "../../../types";
import type { ProjectTab } from "../../../types/project";

function makeFile(overrides: Partial<DiscoveredFile> = {}): DiscoveredFile {
  return {
    path: "./1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
    filename: "4Dell Latitude CPi.E01",
    container_type: "EnCase (E01)",
    size: 671_094_597,
    segment_count: 1,
    ...overrides,
  };
}

function makeCaseDoc(overrides: Partial<CaseDocument> = {}): CaseDocument {
  return {
    path: "/cases/1827-1001/4.Case.Documents/Chain of Custody Form 7-01.pdf",
    filename: "Chain of Custody Form 7-01.pdf",
    size: 2048,
    document_type: "chain_of_custody",
    format: "pdf",
    ...overrides,
  } as CaseDocument;
}

describe("projectLoader", () => {
  it("restores legacy entry tabs from file_path plus entry_path", () => {
    const tabs: ProjectTab[] = [
      {
        id: "entry:/Partition1_NTFS/pagefile.sys",
        type: "entry",
        file_path: "./1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
        entry_path: "/Partition1_NTFS/pagefile.sys",
        name: "pagefile.sys",
        subtitle: "4Dell Latitude CPi.E01",
        order: 0,
      },
    ];

    const restored = restoreCenterTabs(tabs, [makeFile()], [], []);

    expect(restored).toHaveLength(1);
    expect(restored[0]).toMatchObject({
      id: "entry:/Partition1_NTFS/pagefile.sys",
      type: "entry",
      title: "pagefile.sys",
      subtitle: "4Dell Latitude CPi.E01",
    });
    expect(restored[0].entry).toMatchObject({
      containerPath: "./1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
      entryPath: "/Partition1_NTFS/pagefile.sys",
      name: "pagefile.sys",
      isVfsEntry: true,
      isArchiveEntry: false,
      containerType: "EnCase (E01)",
    });
  });

  it("restores document tabs with viewer entries", () => {
    const doc = makeCaseDoc();
    const tabs: ProjectTab[] = [
      {
        id: `document:${doc.path}`,
        type: "document",
        file_path: doc.path,
        document_path: doc.path,
        name: doc.filename,
        order: 0,
      },
    ];

    const restored = restoreCenterTabs(tabs, [], [], [doc]);

    expect(restored).toHaveLength(1);
    expect(restored[0]).toMatchObject({
      id: `document:${doc.path}`,
      type: "document",
      documentPath: doc.path,
      documentEntry: {
        containerPath: doc.path,
        entryPath: doc.path,
        name: doc.filename,
        isDiskFile: true,
      },
    });
  });

  it("shows browser file picker and cancel feedback when Open Project has no selection", async () => {
    const setScanDir = vi.fn();
    const toast = {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    };

    await handleLoadProject({
      fileManager: {
        scanDir: () => "/previous/evidence",
        setScanDir,
      },
      hashManager: {},
      projectManager: {
        loadProject: vi.fn().mockResolvedValue({ project: null, error: "Open cancelled" }),
      },
      processedDbManager: {},
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
      setCenterTabs: vi.fn(),
      setActiveTabId: vi.fn(),
      setCenterViewMode: vi.fn(),
      toast,
    } as any);

    expect(setScanDir).toHaveBeenCalledWith("");
    expect(setScanDir).toHaveBeenCalledWith("/previous/evidence");
    expect(toast.info).toHaveBeenCalledWith(
      "Open Project",
      "Choose a .cffx project file in the browser file picker.",
    );
    expect(toast.info).toHaveBeenCalledWith(
      "Open Cancelled",
      "No project file was selected. Use Open Project and choose a .cffx file.",
    );
  });

  it("does not scan evidence directories after a browser project load without cache", async () => {
    const setScanDir = vi.fn();
    const scanForFiles = vi.fn().mockRejectedValue(new Error("desktop-only scan"));
    const toast = {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    };

    await handleLoadProject({
      fileManager: {
        scanDir: () => "",
        setScanDir,
        scanForFiles,
        restoreDiscoveredFiles: vi.fn(),
        restoreFileInfoMap: vi.fn(),
        setTypeFilter: vi.fn(),
        setActiveFile: vi.fn(),
      },
      hashManager: {
        restoreFileHashMap: vi.fn(),
        restoreHashHistory: vi.fn(),
      },
      projectManager: {
        loadProject: vi.fn().mockResolvedValue({
          project: {
            name: "Browser Project",
            root_path: "/cases/browser",
            tabs: [],
            hash_history: { files: {} },
            evidence_cache: undefined,
            processed_databases: undefined,
          },
          warnings: [],
        }),
        updateLocations: vi.fn(),
      },
      processedDbManager: {
        databases: () => [],
        restoreFullState: vi.fn(),
        restoreFromProject: vi.fn(),
      },
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
      setCenterTabs: vi.fn(),
      setActiveTabId: vi.fn(),
      setCenterViewMode: vi.fn(),
      toast,
    } as any);

    expect(scanForFiles).not.toHaveBeenCalled();
    expect(setScanDir).toHaveBeenCalledWith("/cases/browser");
    expect(toast.info).toHaveBeenCalledWith(
      "Browser Preview",
      "Loaded project metadata. Evidence scanning is available in the desktop app.",
    );
    expect(toast.success).toHaveBeenCalledWith("Project Loaded", "Opened: Browser Project");
    expect(toast.error).not.toHaveBeenCalled();
  });

  it("restores cached documents and processed databases before rebuilding saved tabs", async () => {
    const doc = makeCaseDoc();
    const processedDb = {
      path: "/cases/1827-1001/2.Processed.Database/AXIOM - Nov 15 2025 164907",
      name: "AXIOM - Nov 15 2025 164907",
      type: "axiom",
    };
    let restoredDatabases: any[] = [];
    const setCaseDocuments = vi.fn();
    const setCenterTabs = vi.fn();

    await handleLoadProject({
      fileManager: {
        scanDir: () => "",
        setScanDir: vi.fn(),
        scanForFiles: vi.fn(),
        restoreDiscoveredFiles: vi.fn(),
        restoreFileInfoMap: vi.fn(),
        setTypeFilter: vi.fn(),
        setActiveFile: vi.fn(),
        discoveredFiles: () => [],
      },
      hashManager: {
        restoreFileHashMap: vi.fn(),
        restoreHashHistory: vi.fn(),
      },
      projectManager: {
        loadProject: vi.fn().mockResolvedValue({
          project: {
            name: "Seed Project",
            root_path: "/cases/1827-1001/1.Evidence",
            tabs: [
              {
                id: `document:${doc.path}`,
                type: "document",
                file_path: doc.path,
                document_path: doc.path,
                name: doc.filename,
                order: 0,
              },
              {
                id: `processed:${processedDb.path}`,
                type: "processed",
                file_path: processedDb.path,
                processed_db_path: processedDb.path,
                name: processedDb.name,
                processed_db_type: processedDb.type,
                order: 1,
              },
            ],
            hash_history: { files: {} },
            evidence_cache: undefined,
            processed_databases: {
              cached_databases: [processedDb],
              selected_path: processedDb.path,
            },
            case_documents_cache: {
              valid: true,
              search_path: "/cases/1827-1001/4.Case.Documents",
              documents: [doc],
            },
          },
          warnings: [],
        }),
        updateLocations: vi.fn(),
      },
      processedDbManager: {
        databases: () => restoredDatabases,
        restoreFullState: vi.fn((dbs: any[]) => {
          restoredDatabases = dbs;
        }),
        restoreFromProject: vi.fn(),
      },
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
      setCaseDocuments,
      setCenterTabs,
      setActiveTabId: vi.fn(),
      setCenterViewMode: vi.fn(),
      toast: {
        success: vi.fn(),
        error: vi.fn(),
        warning: vi.fn(),
        info: vi.fn(),
      },
    } as any);

    expect(setCaseDocuments).toHaveBeenCalledWith([doc]);
    expect(setCenterTabs).toHaveBeenCalledWith([
      expect.objectContaining({
        type: "document",
        documentPath: doc.path,
        documentEntry: expect.objectContaining({
          containerPath: doc.path,
          entryPath: doc.path,
        }),
      }),
      expect.objectContaining({
        type: "processed",
        processedDb,
      }),
    ]);
  });
});
