// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  buildSaveOptions,
  createDocumentEntry,
  type BuildSaveOptionsParams,
} from "../project/projectHelpers";
import type { DiscoveredFile, CaseDocument } from "../../types";
import type { SelectedEntry } from "../../components/EvidenceTree";

/**
 * Tests for pure / mostly-pure functions in projectHelpers.ts
 *
 * - buildSaveOptions: gathers state from accessors → returns save payload
 * - createDocumentEntry: maps CaseDocument → SelectedEntry
 */

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Create a minimal mock of BuildSaveOptionsParams with sensible defaults */
function makeParams(overrides: Partial<BuildSaveOptionsParams> = {}): BuildSaveOptionsParams {
  const accessor = <T>(val: T) => () => val;

  const file: DiscoveredFile = {
    path: "/evidence/case.e01",
    filename: "case.e01",
    container_type: "E01",
    size: 1024,
    modified: "2024-01-01",
  } as DiscoveredFile;

  return {
    fileManager: {
      scanDir: accessor("/evidence"),
      activeFile: accessor(file),
      discoveredFiles: accessor([file]),
      fileInfoMap: accessor(new Map()),
      typeFilter: accessor(null),
    } as any,
    hashManager: {
      hashHistory: accessor(new Map()),
      fileHashMap: accessor(new Map()),
    } as any,
    processedDbManager: {
      databases: accessor([]),
      selectedDatabase: accessor(null),
      axiomCaseInfo: accessor({}),
      artifactCategories: accessor({}),
      detailView: accessor({ type: "case" }),
    } as any,
    openTabs: accessor([]),
    centerTabs: accessor([]),
    activeTabId: accessor(null),
    viewMode: accessor("info" as const),
    selectedContainerEntry: accessor(null),
    leftWidth: accessor(320),
    rightWidth: accessor(300),
    leftCollapsed: accessor(false),
    rightCollapsed: accessor(false),
    leftPanelTab: accessor("evidence" as const),
    currentViewMode: accessor("info" as const),
    entryContentViewMode: accessor("auto" as const),
    caseDocumentsPath: accessor(null),
    treeExpansionState: accessor(null),
    caseDocuments: accessor(null),
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// buildSaveOptions
// ---------------------------------------------------------------------------

describe("buildSaveOptions", () => {
  it("returns null when no scan directory is set", () => {
    const params = makeParams({
      fileManager: {
        scanDir: () => null,
        activeFile: () => null,
        discoveredFiles: () => [],
        fileInfoMap: () => new Map(),
        typeFilter: () => null,
      } as any,
    });

    expect(buildSaveOptions(params)).toBeNull();
  });

  it("returns an object when scanDir is set", () => {
    const result = buildSaveOptions(makeParams());
    expect(result).not.toBeNull();
    expect(result!.rootPath).toBe("/evidence");
  });

  it("captures panel dimensions in uiState", () => {
    const result = buildSaveOptions(
      makeParams({
        leftWidth: () => 400,
        rightWidth: () => 250,
        leftCollapsed: () => true,
        rightCollapsed: () => false,
      }),
    );

    expect(result!.uiState.left_panel_width).toBe(400);
    expect(result!.uiState.right_panel_width).toBe(250);
    expect(result!.uiState.left_panel_collapsed).toBe(true);
    expect(result!.uiState.right_panel_collapsed).toBe(false);
  });

  it("captures left panel tab and view modes", () => {
    const result = buildSaveOptions(
      makeParams({
        leftPanelTab: () => "bookmarks",
        currentViewMode: () => "hex",
        entryContentViewMode: () => "document",
      }),
    );

    expect(result!.uiState.left_panel_tab).toBe("bookmarks");
    expect(result!.uiState.detail_view_mode).toBe("hex");
    expect(result!.uiState.entry_content_view_mode).toBe("document");
  });

  it("captures selected container entry", () => {
    const entry: SelectedEntry = {
      containerPath: "/evidence/case.e01",
      entryPath: "/evidence/case.e01/file.txt",
      name: "file.txt",
      size: 512,
      isDir: false,
    } as SelectedEntry;

    const result = buildSaveOptions(
      makeParams({ selectedContainerEntry: () => entry }),
    );

    expect(result!.uiState.selected_entry).toEqual({
      containerPath: "/evidence/case.e01",
      entryPath: "/evidence/case.e01/file.txt",
      name: "file.txt",
    });
  });

  it("sets selected_entry to null when no entry selected", () => {
    const result = buildSaveOptions(
      makeParams({ selectedContainerEntry: () => null }),
    );
    expect(result!.uiState.selected_entry).toBeNull();
  });

  it("captures active file path", () => {
    const file = {
      path: "/evidence/disk.e01",
      filename: "disk.e01",
    } as DiscoveredFile;

    const result = buildSaveOptions(
      makeParams({
        fileManager: {
          scanDir: () => "/evidence",
          activeFile: () => file,
          discoveredFiles: () => [file],
          fileInfoMap: () => new Map(),
          typeFilter: () => null,
        } as any,
      }),
    );

    expect(result!.activeTabPath).toBe("/evidence/disk.e01");
  });

  it("sets activeTabPath to null when no active file", () => {
    const result = buildSaveOptions(
      makeParams({
        fileManager: {
          scanDir: () => "/evidence",
          activeFile: () => null,
          discoveredFiles: () => [],
          fileInfoMap: () => new Map(),
          typeFilter: () => null,
        } as any,
      }),
    );
    expect(result!.activeTabPath).toBeNull();
  });

  it("captures hash history from hashManager", () => {
    const history = new Map([
      ["/evidence/case.e01", [{ algorithm: "SHA-256", hash: "abc123", timestamp: new Date(), source: "computed" as const }]],
    ]);

    const result = buildSaveOptions(
      makeParams({
        hashManager: {
          hashHistory: () => history,
          fileHashMap: () => new Map(),
        } as any,
      }),
    );

    expect(result!.hashHistory).toBe(history);
  });

  it("converts center tabs to serializable format", () => {
    const file = { path: "/a.e01", filename: "a.e01", container_type: "E01" } as DiscoveredFile;
    const centerTabs = [
      { id: "evidence:/a.e01", type: "evidence", title: "a.e01", subtitle: "E01", file, closable: true },
    ];

    const result = buildSaveOptions(
      makeParams({ centerTabs: () => centerTabs as any }),
    );

    expect(result!.centerTabs).toHaveLength(1);
    expect(result!.centerTabs![0]).toEqual({
      id: "evidence:/a.e01",
      type: "evidence",
      title: "a.e01",
      subtitle: "E01",
      file,
      documentPath: undefined,
      entry: undefined,
      processedDb: undefined,
    });
  });

  it("falls back to legacy openTabs when no centerTabs", () => {
    const legacyTab = { file: { path: "/b.e01" }, id: "/b.e01" };

    const result = buildSaveOptions(
      makeParams({
        centerTabs: () => [],
        openTabs: () => [legacyTab] as any,
      }),
    );

    expect(result!.openTabs).toHaveLength(1);
    expect(result!.centerTabs).toBeUndefined();
  });

  it("captures type filter in filterState", () => {
    const result = buildSaveOptions(
      makeParams({
        fileManager: {
          scanDir: () => "/evidence",
          activeFile: () => null,
          discoveredFiles: () => [],
          fileInfoMap: () => new Map(),
          typeFilter: () => "E01",
        } as any,
      }),
    );

    expect(result!.filterState.type_filter).toBe("E01");
  });

  it("includes evidence cache with discovered files and maps", () => {
    const file = { path: "/a.e01" } as DiscoveredFile;
    const infoMap = new Map([["//a.e01", { format: "E01" }]]);
    const hashMap = new Map([["//a.e01", { algorithm: "SHA-256", hash: "abc" }]]);

    const result = buildSaveOptions(
      makeParams({
        fileManager: {
          scanDir: () => "/evidence",
          activeFile: () => file,
          discoveredFiles: () => [file],
          fileInfoMap: () => infoMap,
          typeFilter: () => null,
        } as any,
        hashManager: {
          hashHistory: () => new Map(),
          fileHashMap: () => hashMap,
        } as any,
      }),
    );

    expect(result!.evidenceCache.discoveredFiles).toEqual([file]);
    expect(result!.evidenceCache.fileInfoMap).toBe(infoMap);
    expect(result!.evidenceCache.fileHashMap).toBe(hashMap);
  });

  it("includes case documents cache when documents exist", () => {
    const docs: CaseDocument[] = [
      { path: "/docs/report.pdf", filename: "report.pdf", size: 1024 } as CaseDocument,
    ];

    const result = buildSaveOptions(
      makeParams({
        caseDocuments: () => docs,
        caseDocumentsPath: () => "/docs",
      }),
    );

    expect(result!.caseDocumentsCache).toBeDefined();
    expect(result!.caseDocumentsCache!.documents).toEqual(docs);
    expect(result!.caseDocumentsCache!.searchPath).toBe("/docs");
  });

  it("omits case documents cache when no documents", () => {
    const result = buildSaveOptions(
      makeParams({ caseDocuments: () => null }),
    );
    expect(result!.caseDocumentsCache).toBeUndefined();
  });

  it("falls back searchPath to scanDir when caseDocumentsPath is null", () => {
    const docs = [{ path: "/docs/report.pdf", filename: "report.pdf", size: 1024 } as CaseDocument];

    const result = buildSaveOptions(
      makeParams({
        caseDocuments: () => docs,
        caseDocumentsPath: () => null,
      }),
    );

    expect(result!.caseDocumentsCache!.searchPath).toBe("/evidence");
  });

  it("captures tree expansion state in uiState", () => {
    const treeState = { expanded: new Set(["node1", "node2"]) };

    const result = buildSaveOptions(
      makeParams({ treeExpansionState: () => treeState as any }),
    );

    expect(result!.uiState.tree_expansion_state).toBe(treeState);
  });

  it("captures processed database cache", () => {
    const result = buildSaveOptions(
      makeParams({
        processedDbManager: {
          databases: () => [{ path: "/db.sqlite", name: "Test" }],
          selectedDatabase: () => null,
          axiomCaseInfo: () => ({ "/db.sqlite": { case_number: "123" } }),
          artifactCategories: () => ({}),
          detailView: () => ({ type: "artifacts" }),
        } as any,
      }),
    );

    expect(result!.processedDbCache.databases).toHaveLength(1);
    expect(result!.processedDbCache.axiomCaseInfo).toHaveProperty("/db.sqlite");
    expect(result!.processedDbCache.detailViewType).toBe("artifacts");
  });

  it("captures activeTabId and viewMode", () => {
    const result = buildSaveOptions(
      makeParams({
        activeTabId: () => "evidence:/a.e01",
        viewMode: () => "hex" as const,
      }),
    );

    expect(result!.activeTabId).toBe("evidence:/a.e01");
    expect(result!.viewMode).toBe("hex");
  });
});

// ---------------------------------------------------------------------------
// createDocumentEntry
// ---------------------------------------------------------------------------

describe("createDocumentEntry", () => {
  it("creates a SelectedEntry from a CaseDocument", () => {
    const doc: CaseDocument = {
      path: "/docs/report.pdf",
      filename: "report.pdf",
      size: 2048,
      document_type: "ExternalReport",
      format: "pdf",
    } as CaseDocument;

    const entry = createDocumentEntry(doc);

    expect(entry.containerPath).toBe("/docs/report.pdf");
    expect(entry.entryPath).toBe("/docs/report.pdf");
    expect(entry.name).toBe("report.pdf");
    expect(entry.size).toBe(2048);
    expect(entry.isDir).toBe(false);
    expect(entry.isDiskFile).toBe(true);
    expect(entry.containerType).toBe("pdf");
  });

  it("defaults isDiskFile to true", () => {
    const doc = { path: "/x", filename: "x", size: 0, format: "txt" } as CaseDocument;
    expect(createDocumentEntry(doc).isDiskFile).toBe(true);
  });

  it("accepts isDiskFile override", () => {
    const doc = { path: "/x", filename: "x", size: 0, format: "txt" } as CaseDocument;
    expect(createDocumentEntry(doc, false).isDiskFile).toBe(false);
  });

  it("falls back containerType to 'file' when format is undefined", () => {
    const doc = { path: "/x", filename: "x", size: 100 } as CaseDocument;
    expect(createDocumentEntry(doc).containerType).toBe("file");
  });

  it("includes metadata with document_type and format", () => {
    const doc: CaseDocument = {
      path: "/docs/letter.docx",
      filename: "letter.docx",
      size: 4096,
      document_type: "CaseNotes",
      format: "docx",
      case_number: "2024-001",
    } as CaseDocument;

    const entry = createDocumentEntry(doc);

    expect(entry.metadata).toBeDefined();
    expect(entry.metadata!.document_type).toBe("CaseNotes");
    expect(entry.metadata!.format).toBe("docx");
    expect(entry.metadata!.case_number).toBe("2024-001");
  });
});
