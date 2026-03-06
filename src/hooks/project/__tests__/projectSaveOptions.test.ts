// =============================================================================
// projectSaveOptions — buildSaveOptions tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { buildSaveOptions } from "../projectSaveOptions";
import type { BuildSaveOptionsParams } from "../projectSaveOptions";

/** Create a minimal mock params object. Accessors are trivial functions. */
function makeParams(
  overrides: Partial<Record<string, unknown>> = {},
): BuildSaveOptionsParams {
  const fileManager = {
    scanDir: () => "scanDir" in overrides ? overrides.scanDir : "/evidence",
    activeFile: () => overrides.activeFile ?? null,
    discoveredFiles: () => overrides.discoveredFiles ?? [],
    fileInfoMap: () => overrides.fileInfoMap ?? {},
    typeFilter: () => overrides.typeFilter ?? null,
  };

  const hashManager = {
    hashHistory: () => overrides.hashHistory ?? {},
    fileHashMap: () => overrides.fileHashMap ?? {},
  };

  const processedDbManager = {
    databases: () => overrides.databases ?? [],
    selectedDatabase: () => overrides.selectedDatabase ?? null,
    axiomCaseInfo: () => overrides.axiomCaseInfo ?? null,
    artifactCategories: () => overrides.artifactCategories ?? [],
    detailView: () => overrides.detailView ?? null,
  };

  return {
    fileManager,
    hashManager,
    processedDbManager,
    selectedContainerEntry: () => (overrides.selectedEntry as any) ?? null,
    leftWidth: () => (overrides.leftWidth as number) ?? 300,
    rightWidth: () => (overrides.rightWidth as number) ?? 280,
    leftCollapsed: () => (overrides.leftCollapsed as boolean) ?? false,
    rightCollapsed: () => (overrides.rightCollapsed as boolean) ?? true,
    leftPanelTab: () => (overrides.leftPanelTab as any) ?? "evidence",
    currentViewMode: () => (overrides.currentViewMode as any) ?? "info",
    entryContentViewMode: () =>
      (overrides.entryContentViewMode as any) ?? "auto",
    caseDocumentsPath: () => (overrides.caseDocumentsPath as any) ?? null,
    treeExpansionState: () =>
      (overrides.treeExpansionState as any) ?? null,
    caseDocuments: () => (overrides.caseDocuments as any) ?? null,
    centerTabs: () => (overrides.centerTabs as any) ?? [],
    activeTabId: () => (overrides.activeTabId as any) ?? null,
    viewMode: () => (overrides.viewMode as any) ?? "info",
  } as any;
}

describe("buildSaveOptions", () => {
  it("returns null when scanDir is empty", () => {
    expect(buildSaveOptions(makeParams({ scanDir: "" }))).toBeNull();
  });

  it("returns null when scanDir is undefined", () => {
    expect(
      buildSaveOptions(makeParams({ scanDir: undefined })),
    ).toBeNull();
  });

  it("sets rootPath from scanDir", () => {
    const result = buildSaveOptions(makeParams({ scanDir: "/data" }));
    expect(result?.rootPath).toBe("/data");
  });

  it("captures centerTabs", () => {
    const tabs = [
      {
        id: "tab-1",
        type: "evidence",
        title: "E01 File",
        subtitle: undefined,
        file: undefined,
        documentPath: undefined,
        entry: undefined,
        processedDb: undefined,
        collectionId: undefined,
        collectionReadOnly: undefined,
        collectionListView: undefined,
      },
    ];
    const result = buildSaveOptions(makeParams({ centerTabs: tabs }));
    expect(result?.centerTabs).toHaveLength(1);
    expect(result?.centerTabs![0].id).toBe("tab-1");
  });

  it("falls back to legacy openTabs when centerTabs is empty", () => {
    const result = buildSaveOptions(makeParams({ centerTabs: [] }));
    expect(result?.openTabs).toEqual([]);
    expect(result?.centerTabs).toBeUndefined();
  });

  it("captures selectedEntry data", () => {
    const entry = {
      containerPath: "/img.e01",
      entryPath: "Documents/readme.txt",
      name: "readme.txt",
    };
    const result = buildSaveOptions(makeParams({ selectedEntry: entry }));
    expect(result?.uiState.selected_entry).toEqual(entry);
  });

  it("sets selected_entry to null when no entry", () => {
    const result = buildSaveOptions(makeParams());
    expect(result?.uiState.selected_entry).toBeNull();
  });

  it("captures uiState dimensions", () => {
    const result = buildSaveOptions(
      makeParams({ leftWidth: 400, rightWidth: 350 }),
    );
    expect(result?.uiState.left_panel_width).toBe(400);
    expect(result?.uiState.right_panel_width).toBe(350);
  });

  it("captures filter state", () => {
    const result = buildSaveOptions(makeParams({ typeFilter: "ad1" }));
    expect(result?.filterState.type_filter).toBe("ad1");
    expect(result?.filterState.sort_by).toBe("name");
  });

  it("captures evidence cache", () => {
    const files = [{ path: "/a.e01", name: "a.e01" }];
    const result = buildSaveOptions(
      makeParams({ discoveredFiles: files }),
    );
    expect(result?.evidenceCache.discoveredFiles).toEqual(files);
  });

  it("captures processed database cache", () => {
    const dbs = [{ id: "db-1" }];
    const result = buildSaveOptions(
      makeParams({ databases: dbs }),
    );
    expect(result?.processedDbCache.databases).toEqual(dbs);
  });

  it("includes caseDocumentsCache when documents exist", () => {
    const docs = [{ path: "/doc.pdf", filename: "doc.pdf" }];
    const result = buildSaveOptions(
      makeParams({ caseDocuments: docs, caseDocumentsPath: "/docs" }),
    );
    expect(result?.caseDocumentsCache?.documents).toEqual(docs);
    expect(result?.caseDocumentsCache?.searchPath).toBe("/docs");
  });

  it("omits caseDocumentsCache when documents are null", () => {
    const result = buildSaveOptions(
      makeParams({ caseDocuments: null }),
    );
    expect(result?.caseDocumentsCache).toBeUndefined();
  });
});
