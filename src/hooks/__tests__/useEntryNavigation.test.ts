// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot } from "solid-js";
import { useEntryNavigation, type UseEntryNavigationDeps } from "../useEntryNavigation";
import type { SelectedEntry } from "../../components/EvidenceTree";
import type { DiscoveredFile, CaseDocument } from "../../types";

/** Run a test inside a SolidJS reactive root. */
function testWithRoot<T>(fn: () => T): T {
  let result!: T;
  createRoot((dispose) => {
    result = fn();
    dispose();
  });
  return result;
}

/** Create mock dependencies for useEntryNavigation. */
function createMockDeps(): UseEntryNavigationDeps & {
  mocks: {
    selectAndViewFile: ReturnType<typeof vi.fn>;
    addDiscoveredFile: ReturnType<typeof vi.fn>;
    setActiveFile: ReturnType<typeof vi.fn>;
    openContainerEntry: ReturnType<typeof vi.fn>;
    openEvidenceFile: ReturnType<typeof vi.fn>;
    openCaseDocument: ReturnType<typeof vi.fn>;
    openProcessedDatabase: ReturnType<typeof vi.fn>;
    selectDatabase: ReturnType<typeof vi.fn>;
    setSelectedContainerEntry: ReturnType<typeof vi.fn>;
    setEntryContentViewMode: ReturnType<typeof vi.fn>;
    toastSuccess: ReturnType<typeof vi.fn>;
  };
} {
  const mocks = {
    selectAndViewFile: vi.fn(),
    addDiscoveredFile: vi.fn(),
    setActiveFile: vi.fn(),
    openContainerEntry: vi.fn(),
    openEvidenceFile: vi.fn(),
    openCaseDocument: vi.fn(),
    openProcessedDatabase: vi.fn(),
    selectDatabase: vi.fn(),
    setSelectedContainerEntry: vi.fn(),
    setEntryContentViewMode: vi.fn(),
    toastSuccess: vi.fn(),
  };

  return {
    fileManager: {
      selectAndViewFile: mocks.selectAndViewFile,
      addDiscoveredFile: mocks.addDiscoveredFile,
      setActiveFile: mocks.setActiveFile,
    },
    centerPaneTabs: {
      openContainerEntry: mocks.openContainerEntry,
      openEvidenceFile: mocks.openEvidenceFile,
      openCaseDocument: mocks.openCaseDocument,
      openProcessedDatabase: mocks.openProcessedDatabase,
    },
    processedDbManager: {
      selectDatabase: mocks.selectDatabase,
    },
    setSelectedContainerEntry: mocks.setSelectedContainerEntry as any,
    setEntryContentViewMode: mocks.setEntryContentViewMode as any,
    toast: { success: mocks.toastSuccess },
    mocks,
  };
}

/** Create a minimal SelectedEntry for testing. */
function makeEntry(overrides: Partial<SelectedEntry> = {}): SelectedEntry {
  return {
    containerPath: overrides.containerPath ?? "/evidence/test.ad1",
    entryPath: overrides.entryPath ?? "Documents/readme.txt",
    name: overrides.name ?? "readme.txt",
    size: overrides.size ?? 1024,
    isDir: overrides.isDir ?? false,
    containerType: overrides.containerType ?? "AD1",
    ...overrides,
  };
}

/** Create a minimal DiscoveredFile for testing. */
function makeFile(overrides: Partial<DiscoveredFile> = {}): DiscoveredFile {
  return {
    path: overrides.path ?? "/evidence/test.e01",
    filename: overrides.filename ?? "test.e01",
    container_type: overrides.container_type ?? "E01",
    size: overrides.size ?? 1048576,
    segment_count: overrides.segment_count ?? 1,
    ...overrides,
  };
}

/** Create a minimal CaseDocument for testing. */
function makeCaseDoc(overrides: Partial<CaseDocument> = {}): CaseDocument {
  return {
    filename: overrides.filename ?? "warrant.pdf",
    path: overrides.path ?? "/case/docs/warrant.pdf",
    document_type: overrides.document_type ?? "Other",
    size: overrides.size ?? 2048,
    format: overrides.format ?? "pdf",
    modified: overrides.modified ?? "2024-01-15T10:30:00Z",
    ...overrides,
  };
}

describe("useEntryNavigation", () => {
  let deps: ReturnType<typeof createMockDeps>;

  beforeEach(() => {
    deps = createMockDeps();
  });

  // ---------------------------------------------------------------------------
  // Return shape
  // ---------------------------------------------------------------------------
  describe("return shape", () => {
    it("returns all 7 handler functions", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      expect(nav.handleSelectEntry).toBeTypeOf("function");
      expect(nav.handleSelectEvidenceFile).toBeTypeOf("function");
      expect(nav.handleOpenNestedContainer).toBeTypeOf("function");
      expect(nav.handleSelectProcessedDb).toBeTypeOf("function");
      expect(nav.handleCaseDocumentSelect).toBeTypeOf("function");
      expect(nav.handleCaseDocViewHex).toBeTypeOf("function");
      expect(nav.handleCaseDocViewText).toBeTypeOf("function");
    });
  });

  // ---------------------------------------------------------------------------
  // handleSelectEntry
  // ---------------------------------------------------------------------------
  describe("handleSelectEntry", () => {
    it("sets selected container entry via legacy setter", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const entry = makeEntry();
      nav.handleSelectEntry(entry);
      expect(deps.mocks.setSelectedContainerEntry).toHaveBeenCalledWith(entry);
    });

    it("opens entry in center pane tab", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const entry = makeEntry();
      nav.handleSelectEntry(entry);
      expect(deps.mocks.openContainerEntry).toHaveBeenCalledWith(entry);
    });

    it("sets entry content view mode to 'auto'", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleSelectEntry(makeEntry());
      expect(deps.mocks.setEntryContentViewMode).toHaveBeenCalledWith("auto");
    });

    it("calls all three dependencies in order (setMode before openTab for correct initial viewMode)", () => {
      const callOrder: string[] = [];
      deps.mocks.setSelectedContainerEntry.mockImplementation(() => callOrder.push("setEntry"));
      deps.mocks.openContainerEntry.mockImplementation(() => callOrder.push("openTab"));
      deps.mocks.setEntryContentViewMode.mockImplementation(() => callOrder.push("setMode"));

      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleSelectEntry(makeEntry());
      expect(callOrder).toEqual(["setEntry", "setMode", "openTab"]);
    });
  });

  // ---------------------------------------------------------------------------
  // handleSelectEvidenceFile
  // ---------------------------------------------------------------------------
  describe("handleSelectEvidenceFile", () => {
    it("calls selectAndViewFile on file manager", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const file = makeFile();
      nav.handleSelectEvidenceFile(file);
      expect(deps.mocks.selectAndViewFile).toHaveBeenCalledWith(file);
    });

    it("opens evidence file in center pane tab", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const file = makeFile();
      nav.handleSelectEvidenceFile(file);
      expect(deps.mocks.openEvidenceFile).toHaveBeenCalledWith(file);
    });
  });

  // ---------------------------------------------------------------------------
  // handleOpenNestedContainer
  // ---------------------------------------------------------------------------
  describe("handleOpenNestedContainer", () => {
    it("adds a discovered file with correct metadata", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleOpenNestedContainer(
        "/tmp/nested.ad1",
        "inner.ad1",
        "ad1",
        "/evidence/outer.e01",
      );

      expect(deps.mocks.addDiscoveredFile).toHaveBeenCalledTimes(1);
      const addedFile = deps.mocks.addDiscoveredFile.mock.calls[0][0] as DiscoveredFile;
      expect(addedFile.path).toBe("/tmp/nested.ad1");
      expect(addedFile.container_type).toBe("AD1"); // uppercased
      expect(addedFile.size).toBe(0);
      expect(addedFile.segment_count).toBe(1);
    });

    it("formats filename with emoji and parent info", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleOpenNestedContainer(
        "/tmp/nested.ad1",
        "inner.ad1",
        "ad1",
        "/evidence/outer.e01",
      );

      const addedFile = deps.mocks.addDiscoveredFile.mock.calls[0][0] as DiscoveredFile;
      expect(addedFile.filename).toBe("📦 inner.ad1 (from outer.e01)");
    });

    it("uses full parent path when no slash in parent", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleOpenNestedContainer("/tmp/x", "child.zip", "zip", "parentonly");

      const addedFile = deps.mocks.addDiscoveredFile.mock.calls[0][0] as DiscoveredFile;
      expect(addedFile.filename).toBe("📦 child.zip (from parentonly)");
    });

    it("selects and views the nested file", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleOpenNestedContainer("/tmp/nested.ad1", "inner.ad1", "ad1", "/evidence/outer.e01");

      expect(deps.mocks.selectAndViewFile).toHaveBeenCalledTimes(1);
      expect(deps.mocks.openEvidenceFile).toHaveBeenCalledTimes(1);
    });

    it("shows toast notification", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleOpenNestedContainer("/tmp/nested.ad1", "inner.ad1", "ad1", "/evidence/outer.e01");

      expect(deps.mocks.toastSuccess).toHaveBeenCalledWith("Nested Container", "Opened inner.ad1");
    });

    it("uppercases container type", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleOpenNestedContainer("/tmp/x", "test.7z", "7z", "/parent");

      const addedFile = deps.mocks.addDiscoveredFile.mock.calls[0][0] as DiscoveredFile;
      expect(addedFile.container_type).toBe("7Z");
    });
  });

  // ---------------------------------------------------------------------------
  // handleSelectProcessedDb
  // ---------------------------------------------------------------------------
  describe("handleSelectProcessedDb", () => {
    it("selects the database via processedDbManager", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const db = { db_type: "MagnetAxiom" as const, path: "/evidence/axiom.db", name: "axiom.db" };
      nav.handleSelectProcessedDb(db);
      expect(deps.mocks.selectDatabase).toHaveBeenCalledWith(db);
    });

    it("clears the active file", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleSelectProcessedDb({ db_type: "GenericSqlite" as const, path: "/test.db", name: "test.db" });
      expect(deps.mocks.setActiveFile).toHaveBeenCalledWith(null);
    });

    it("opens processed database in center pane when db is truthy", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const db = { db_type: "MagnetAxiom" as const, path: "/evidence/axiom.db", name: "axiom.db" };
      nav.handleSelectProcessedDb(db);
      expect(deps.mocks.openProcessedDatabase).toHaveBeenCalledWith(db);
    });

    it("does NOT open center pane tab when db is null", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleSelectProcessedDb(null);
      expect(deps.mocks.selectDatabase).toHaveBeenCalledWith(null);
      expect(deps.mocks.setActiveFile).toHaveBeenCalledWith(null);
      expect(deps.mocks.openProcessedDatabase).not.toHaveBeenCalled();
    });
  });

  // ---------------------------------------------------------------------------
  // handleCaseDocumentSelect
  // ---------------------------------------------------------------------------
  describe("handleCaseDocumentSelect", () => {
    it("opens case document in center pane", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const doc = makeCaseDoc();
      nav.handleCaseDocumentSelect(doc);
      expect(deps.mocks.openCaseDocument).toHaveBeenCalledWith(doc);
    });

    it("sets entry content view mode to 'auto'", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleCaseDocumentSelect(makeCaseDoc());
      expect(deps.mocks.setEntryContentViewMode).toHaveBeenCalledWith("auto");
    });
  });

  // ---------------------------------------------------------------------------
  // handleCaseDocViewHex
  // ---------------------------------------------------------------------------
  describe("handleCaseDocViewHex", () => {
    it("opens case document in center pane", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const doc = makeCaseDoc();
      nav.handleCaseDocViewHex(doc);
      expect(deps.mocks.openCaseDocument).toHaveBeenCalledWith(doc);
    });

    it("sets entry content view mode to 'hex'", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleCaseDocViewHex(makeCaseDoc());
      expect(deps.mocks.setEntryContentViewMode).toHaveBeenCalledWith("hex");
    });
  });

  // ---------------------------------------------------------------------------
  // handleCaseDocViewText
  // ---------------------------------------------------------------------------
  describe("handleCaseDocViewText", () => {
    it("opens case document in center pane", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const doc = makeCaseDoc();
      nav.handleCaseDocViewText(doc);
      expect(deps.mocks.openCaseDocument).toHaveBeenCalledWith(doc);
    });

    it("sets entry content view mode to 'text'", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleCaseDocViewText(makeCaseDoc());
      expect(deps.mocks.setEntryContentViewMode).toHaveBeenCalledWith("text");
    });
  });

  // ---------------------------------------------------------------------------
  // Edge cases
  // ---------------------------------------------------------------------------
  describe("edge cases", () => {
    it("handles entry with empty strings gracefully", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const entry = makeEntry({
        containerPath: "",
        entryPath: "",
        name: "",
      });
      nav.handleSelectEntry(entry);
      expect(deps.mocks.setSelectedContainerEntry).toHaveBeenCalledWith(entry);
      expect(deps.mocks.openContainerEntry).toHaveBeenCalledWith(entry);
    });

    it("handleOpenNestedContainer works with empty parent path", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      nav.handleOpenNestedContainer("/tmp/x", "test.ad1", "ad1", "");

      const addedFile = deps.mocks.addDiscoveredFile.mock.calls[0][0] as DiscoveredFile;
      // Empty string: split('/').pop() returns '', which is falsy → uses full parentPath ''
      expect(addedFile.filename).toContain("test.ad1");
    });

    it("multiple handler calls maintain independence", () => {
      const nav = testWithRoot(() => useEntryNavigation(deps));
      const doc1 = makeCaseDoc({ filename: "a.pdf" });
      const doc2 = makeCaseDoc({ filename: "b.pdf" });

      nav.handleCaseDocViewHex(doc1);
      nav.handleCaseDocViewText(doc2);

      expect(deps.mocks.openCaseDocument).toHaveBeenCalledTimes(2);
      expect(deps.mocks.setEntryContentViewMode).toHaveBeenNthCalledWith(1, "hex");
      expect(deps.mocks.setEntryContentViewMode).toHaveBeenNthCalledWith(2, "text");
    });
  });
});
