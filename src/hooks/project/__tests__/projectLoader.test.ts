// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it, vi } from "vitest";
import { handleLoadProject, restoreCenterTabs, restoreSelectedEntry } from "../projectLoader";
import type { CaseDocument, DiscoveredFile } from "../../../types";
import type { ProjectTab } from "../../../types/project";
import type { SelectedEntry } from "../../../components/EvidenceTree/types";

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
  it("restores selected AD1 entries with saved address metadata", () => {
    const savedEntry: SelectedEntry = {
      containerPath: "/cases/1827-1001/1.Evidence/export.AD1",
      entryPath: "/Users/terry/Documents/report.pdf",
      name: "report.pdf",
      size: 4096,
      isDir: false,
      containerType: "ad1",
      dataAddr: 8192,
      itemAddr: 4096,
      compressedSize: 2048,
      dataEndAddr: 10240,
      metadataAddr: 12288,
      firstChildAddr: null,
      metadata: {
        md5: "abc",
      },
    };

    const restored = restoreSelectedEntry(savedEntry, [
      makeFile({
        path: savedEntry.containerPath,
        filename: "export.AD1",
        container_type: "AccessData (AD1)",
      }),
    ]);

    expect(restored).toEqual({
      ...savedEntry,
      isVfsEntry: false,
      isArchiveEntry: false,
    });
  });

  it("infers selected entry browser flags from cached evidence when old projects omitted them", () => {
    const savedEntry = {
      containerPath: "./1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
      entryPath: "/Partition1_NTFS/pagefile.sys",
      name: "pagefile.sys",
      size: undefined,
      isDir: undefined,
    } as unknown as SelectedEntry;

    const restored = restoreSelectedEntry(savedEntry, [makeFile()]);

    expect(restored).toMatchObject({
      containerPath: savedEntry.containerPath,
      entryPath: savedEntry.entryPath,
      name: "pagefile.sys",
      size: 0,
      isDir: false,
      isVfsEntry: true,
      isArchiveEntry: false,
      containerType: "EnCase (E01)",
    });
  });

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

  it("restores entry tabs with saved size and AD1 address metadata", () => {
    const tabs = [
      {
        id: "entry:/Users/terry/Documents/report.pdf",
        type: "entry",
        file_path: "/cases/1827-1001/1.Evidence/export.AD1",
        entry_container_path: "/cases/1827-1001/1.Evidence/export.AD1",
        entry_path: "/Users/terry/Documents/report.pdf",
        entry_name: "report.pdf",
        name: "report.pdf",
        order: 0,
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
      },
    ] as unknown as ProjectTab[];

    const restored = restoreCenterTabs(
      tabs,
      [
        makeFile({
          path: "/cases/1827-1001/1.Evidence/export.AD1",
          filename: "export.AD1",
          container_type: "AccessData (AD1)",
        }),
      ],
      [],
      [],
    );

    expect(restored).toHaveLength(1);
    expect(restored[0].entry).toMatchObject({
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

  it("normalizes restored evidence tab IDs to the resolved discovered file path", () => {
    const resolvedFile = makeFile({
      path: "/Users/terryreynolds/Cases/1827-1001/1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
    });
    const tabs: ProjectTab[] = [
      {
        id: "evidence:/Users/terryreynolds/Old Case Root/1.Evidence/4Dell Latitude CPi/4Dell Latitude CPi.E01",
        type: "evidence",
        file_path: resolvedFile.path,
        name: "4Dell Latitude CPi.E01",
        container_type: "EnCase (E01)",
        order: 0,
      },
    ];

    const restored = restoreCenterTabs(tabs, [resolvedFile], [], []);

    expect(restored).toHaveLength(1);
    expect(restored[0].id).toBe(`evidence:${resolvedFile.path}`);
    expect(restored[0].file).toBe(resolvedFile);
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

  it("reports a warning when project state loaded but post-load setup reports an error", async () => {
    const loadedProject = {
      name: "26-000",
      root_path: "/Users/terryreynolds/Cases/1827-1001/1.Evidence",
      tabs: [],
      hash_history: { files: {} },
    };
    const toast = {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    };

    await handleLoadProject({
      fileManager: {
        scanDir: () => "",
        setScanDir: vi.fn(),
      },
      hashManager: {},
      projectManager: {
        project: () => loadedProject,
        loadProject: vi.fn().mockResolvedValue({
          project: null,
          error: "Post-load setup warning: audit storage unavailable",
        }),
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

    expect(toast.error).not.toHaveBeenCalled();
    expect(toast.warning).toHaveBeenCalledWith(
      "Project Loaded With Warnings",
      expect.stringContaining("26-000"),
    );
  });

  it("reports thrown post-load failures as warnings when project state is already loaded", async () => {
    const loadedProject = {
      name: "26-000",
      root_path: "/Users/terryreynolds/Cases/1827-1001/1.Evidence",
      tabs: [],
      hash_history: { files: {} },
    };
    const toast = {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    };

    await handleLoadProject({
      fileManager: {
        scanDir: () => "",
        setScanDir: vi.fn(),
      },
      hashManager: {},
      projectManager: {
        project: () => loadedProject,
        loadProject: vi.fn().mockRejectedValue(new Error("audit storage unavailable")),
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

    expect(toast.error).not.toHaveBeenCalled();
    expect(toast.warning).toHaveBeenCalledWith(
      "Project Loaded With Warnings",
      expect.stringContaining("audit storage unavailable"),
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

  it("restores the parent evidence file for an active entry tab", async () => {
    const parentFile = makeFile();
    const setActiveFile = vi.fn();
    const setActiveTabId = vi.fn();

    await handleLoadProject({
      fileManager: {
        scanDir: () => "",
        setScanDir: vi.fn(),
        scanForFiles: vi.fn(),
        restoreDiscoveredFiles: vi.fn(),
        restoreFileInfoMap: vi.fn(),
        setTypeFilter: vi.fn(),
        setActiveFile,
        discoveredFiles: () => [parentFile],
      },
      hashManager: {
        restoreFileHashMap: vi.fn(),
        restoreHashHistory: vi.fn(),
      },
      projectManager: {
        loadProject: vi.fn().mockResolvedValue({
          project: {
            name: "Entry Project",
            root_path: "/cases/1827-1001/1.Evidence",
            tabs: [
              {
                id: "entry:/Partition1_NTFS/pagefile.sys",
                type: "entry",
                file_path: parentFile.path,
                entry_container_path: parentFile.path,
                entry_path: "/Partition1_NTFS/pagefile.sys",
                entry_name: "pagefile.sys",
                name: "pagefile.sys",
                order: 0,
              },
            ],
            active_tab_path: parentFile.path,
            center_pane_state: {
              active_tab_id: "entry:/Partition1_NTFS/pagefile.sys",
              view_mode: "document",
            },
            hash_history: { files: {} },
            evidence_cache: {
              valid: true,
              discovered_files: [parentFile],
              file_info: {},
              computed_hashes: {},
            },
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
      setActiveTabId,
      setCenterViewMode: vi.fn(),
      toast: {
        success: vi.fn(),
        error: vi.fn(),
        warning: vi.fn(),
        info: vi.fn(),
      },
    } as any);

    expect(setActiveTabId).toHaveBeenCalledWith(
      "entry:/Partition1_NTFS/pagefile.sys",
    );
    expect(setActiveFile).toHaveBeenCalledWith(parentFile);
  });

  it("falls back to active_tab_path when center pane active tab id is stale", async () => {
    const firstFile = makeFile({
      path: "/cases/1827-1001/1.Evidence/first.E01",
      filename: "first.E01",
    });
    const secondFile = makeFile({
      path: "/cases/1827-1001/1.Evidence/second.AD1",
      filename: "second.AD1",
      container_type: "AccessData (AD1)",
    });
    const setActiveFile = vi.fn();
    const setActiveTabId = vi.fn();

    await handleLoadProject({
      fileManager: {
        scanDir: () => "",
        setScanDir: vi.fn(),
        scanForFiles: vi.fn(),
        restoreDiscoveredFiles: vi.fn(),
        restoreFileInfoMap: vi.fn(),
        setTypeFilter: vi.fn(),
        setActiveFile,
        discoveredFiles: () => [firstFile, secondFile],
      },
      hashManager: {
        restoreFileHashMap: vi.fn(),
        restoreHashHistory: vi.fn(),
      },
      projectManager: {
        loadProject: vi.fn().mockResolvedValue({
          project: {
            name: "Path Fallback Project",
            root_path: "/cases/1827-1001/1.Evidence",
            tabs: [
              {
                id: `evidence:${firstFile.path}`,
                type: "evidence",
                file_path: firstFile.path,
                name: firstFile.filename,
                order: 0,
              },
              {
                id: `evidence:${secondFile.path}`,
                type: "evidence",
                file_path: secondFile.path,
                name: secondFile.filename,
                order: 1,
              },
            ],
            active_tab_path: secondFile.path,
            center_pane_state: {
              active_tab_id: "missing-active-tab",
              view_mode: "info",
            },
            hash_history: { files: {} },
            evidence_cache: {
              valid: true,
              discovered_files: [firstFile, secondFile],
              file_info: {},
              computed_hashes: {},
            },
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
      setActiveTabId,
      setCenterViewMode: vi.fn(),
      toast: {
        success: vi.fn(),
        error: vi.fn(),
        warning: vi.fn(),
        info: vi.fn(),
      },
    } as any);

    expect(setActiveTabId).toHaveBeenCalledWith(
      `evidence:${secondFile.path}`,
    );
    expect(setActiveFile).toHaveBeenCalledWith(secondFile);
  });

  it("reports restore callback failures as warnings after the project has loaded", async () => {
    const doc = makeCaseDoc();
    const toast = {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    };

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
            locations: {
              evidence_path: "/cases/1827-1001/1.Evidence",
            },
            tabs: [
              {
                id: `document:${doc.path}`,
                type: "document",
                file_path: doc.path,
                document_path: doc.path,
                name: doc.filename,
                order: 0,
              },
            ],
            center_pane_state: {
              active_tab_id: `document:${doc.path}`,
              view_mode: "document",
            },
            hash_history: { files: {} },
            evidence_cache: {
              valid: true,
              discovered_files: [],
              file_info: {},
              computed_hashes: {},
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
      setCenterTabs: vi.fn(() => {
        throw new Error("saved tab restore failed");
      }),
      setActiveTabId: vi.fn(),
      setCenterViewMode: vi.fn(),
      toast,
    } as any);

    expect(toast.error).not.toHaveBeenCalled();
    expect(toast.warning).toHaveBeenCalledWith(
      "Project Loaded With Warnings",
      expect.stringContaining("saved tab restore failed"),
    );
  });
});
