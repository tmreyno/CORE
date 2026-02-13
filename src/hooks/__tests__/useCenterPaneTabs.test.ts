// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { createRoot } from "solid-js";
import { useCenterPaneTabs, type CenterPaneTabsState } from "../useCenterPaneTabs";
import type { DiscoveredFile, CaseDocument, ProcessedDatabase } from "../../types";
import type { SelectedEntry } from "../../components/EvidenceTree";

/**
 * Tests for useCenterPaneTabs — the unified tab management hook.
 *
 * Covers: open/close/dedup/recently-closed tracking for all tab types
 * (evidence, document, entry, processed, export).
 */

// ---------------------------------------------------------------------------
// Mock createDocumentEntry since it's imported from projectHelpers
// ---------------------------------------------------------------------------

vi.mock("../project/projectHelpers", () => ({
  createDocumentEntry: vi.fn((doc: CaseDocument) => ({
    containerPath: doc.path,
    entryPath: doc.path,
    name: doc.filename,
    size: doc.size,
    isDir: false,
    isDiskFile: true,
    containerType: doc.format || "file",
  })),
}));

// ---------------------------------------------------------------------------
// Test data factories
// ---------------------------------------------------------------------------

function makeFile(overrides: Partial<DiscoveredFile> = {}): DiscoveredFile {
  return {
    path: "/evidence/case.e01",
    filename: "case.e01",
    container_type: "E01",
    size: 1024,
    modified: "2024-01-01",
    ...overrides,
  } as DiscoveredFile;
}

function makeCaseDoc(overrides: Partial<CaseDocument> = {}): CaseDocument {
  return {
    path: "/docs/report.pdf",
    filename: "report.pdf",
    size: 2048,
    document_type: "report",
    format: "pdf",
    ...overrides,
  } as CaseDocument;
}

function makeEntry(overrides: Partial<SelectedEntry> = {}): SelectedEntry {
  return {
    containerPath: "/evidence/case.e01",
    entryPath: "/evidence/case.e01/file.txt",
    name: "file.txt",
    size: 512,
    isDir: false,
    isDiskFile: false,
    containerType: "E01",
    ...overrides,
  } as SelectedEntry;
}

function makeProcessedDb(overrides: Partial<ProcessedDatabase> = {}): ProcessedDatabase {
  return {
    path: "/processed/axiom.db",
    name: "Axiom Case",
    case_name: "Case 123",
    db_type: "axiom",
    ...overrides,
  } as ProcessedDatabase;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("useCenterPaneTabs", () => {
  let state: CenterPaneTabsState;
  let dispose: () => void;

  beforeEach(() => {
    createRoot(d => {
      dispose = d;
      state = useCenterPaneTabs();
    });
  });

  afterEach(() => {
    dispose();
  });

  // =========================================================================
  // Initial state
  // =========================================================================

  describe("initial state", () => {
    it("starts with no tabs", () => {
      expect(state.tabs()).toEqual([]);
    });

    it("has no active tab", () => {
      expect(state.activeTabId()).toBeNull();
      expect(state.activeTab()).toBeNull();
      expect(state.activeTabType()).toBeNull();
    });

    it("defaults view mode to info", () => {
      expect(state.viewMode()).toBe("info");
    });

    it("has empty recently-closed set", () => {
      expect(state.recentlyClosed().size).toBe(0);
    });
  });

  // =========================================================================
  // Evidence file tabs
  // =========================================================================

  describe("openEvidenceFile", () => {
    it("creates a tab with correct properties", () => {
      const file = makeFile();
      state.openEvidenceFile(file);

      expect(state.tabs()).toHaveLength(1);
      const tab = state.tabs()[0];
      expect(tab.id).toBe("evidence:/evidence/case.e01");
      expect(tab.type).toBe("evidence");
      expect(tab.title).toBe("case.e01");
      expect(tab.subtitle).toBe("E01");
      expect(tab.file).toBe(file);
      expect(tab.closable).toBe(true);
    });

    it("sets the new tab as active", () => {
      state.openEvidenceFile(makeFile());
      expect(state.activeTabId()).toBe("evidence:/evidence/case.e01");
      expect(state.activeTab()?.type).toBe("evidence");
      expect(state.activeTabType()).toBe("evidence");
    });

    it("sets view mode to info", () => {
      state.setViewMode("hex"); // change from default
      state.openEvidenceFile(makeFile());
      expect(state.viewMode()).toBe("info");
    });

    it("does not duplicate if tab already exists", () => {
      const file = makeFile();
      state.openEvidenceFile(file);
      state.openEvidenceFile(file);

      expect(state.tabs()).toHaveLength(1);
    });

    it("reactivates existing tab instead of creating duplicate", () => {
      const file1 = makeFile({ path: "/a.e01", filename: "a.e01" });
      const file2 = makeFile({ path: "/b.e01", filename: "b.e01" });

      state.openEvidenceFile(file1);
      state.openEvidenceFile(file2);
      expect(state.activeTabId()).toBe("evidence:/b.e01");

      state.openEvidenceFile(file1);
      expect(state.activeTabId()).toBe("evidence:/a.e01");
      expect(state.tabs()).toHaveLength(2);
    });

    it("does not re-open a recently closed tab", () => {
      const file = makeFile();
      state.openEvidenceFile(file);
      state.closeTab("evidence:/evidence/case.e01");

      // Try to open the same file again immediately
      state.openEvidenceFile(file);
      expect(state.tabs()).toHaveLength(0);
    });
  });

  // =========================================================================
  // Case document tabs
  // =========================================================================

  describe("openCaseDocument", () => {
    it("creates a document tab", () => {
      const doc = makeCaseDoc();
      state.openCaseDocument(doc);

      expect(state.tabs()).toHaveLength(1);
      const tab = state.tabs()[0];
      expect(tab.id).toBe("document:/docs/report.pdf");
      expect(tab.type).toBe("document");
      expect(tab.title).toBe("report.pdf");
      expect(tab.documentPath).toBe("/docs/report.pdf");
      expect(tab.documentEntry).toBeDefined();
    });

    it("sets view mode to document", () => {
      state.openCaseDocument(makeCaseDoc());
      expect(state.viewMode()).toBe("document");
    });

    it("deduplicates by path", () => {
      const doc = makeCaseDoc();
      state.openCaseDocument(doc);
      state.openCaseDocument(doc);
      expect(state.tabs()).toHaveLength(1);
    });
  });

  // =========================================================================
  // Container entry tabs
  // =========================================================================

  describe("openContainerEntry", () => {
    it("creates an entry tab", () => {
      const entry = makeEntry();
      state.openContainerEntry(entry);

      expect(state.tabs()).toHaveLength(1);
      const tab = state.tabs()[0];
      expect(tab.id).toBe("entry:/evidence/case.e01/file.txt");
      expect(tab.type).toBe("entry");
      expect(tab.title).toBe("file.txt");
      expect(tab.entry).toBe(entry);
    });

    it("ignores directory entries", () => {
      const dirEntry = makeEntry({ isDir: true, name: "folder" });
      state.openContainerEntry(dirEntry);
      expect(state.tabs()).toHaveLength(0);
    });

    it("sets view mode to document", () => {
      state.openContainerEntry(makeEntry());
      expect(state.viewMode()).toBe("document");
    });

    it("shows container name as subtitle", () => {
      const entry = makeEntry({ containerPath: "/evidence/disk.e01" });
      state.openContainerEntry(entry);
      expect(state.tabs()[0].subtitle).toBe("disk.e01");
    });
  });

  // =========================================================================
  // Processed database tabs
  // =========================================================================

  describe("openProcessedDatabase", () => {
    it("creates a processed tab", () => {
      const db = makeProcessedDb();
      state.openProcessedDatabase(db);

      expect(state.tabs()).toHaveLength(1);
      const tab = state.tabs()[0];
      expect(tab.id).toBe("processed:/processed/axiom.db");
      expect(tab.type).toBe("processed");
      expect(tab.title).toBe("Axiom Case");
      expect(tab.processedDb).toBe(db);
    });

    it("falls back to case_name then Database for title", () => {
      const db1 = makeProcessedDb({ name: undefined, case_name: "Case ABC" });
      state.openProcessedDatabase(db1);
      expect(state.tabs()[0].title).toBe("Case ABC");

      const db2 = makeProcessedDb({
        name: undefined,
        case_name: undefined,
        path: "/other.db",
      });
      state.openProcessedDatabase(db2);
      expect(state.tabs()[1].title).toBe("Database");
    });

    it("does not change view mode (unlike evidence/document)", () => {
      state.setViewMode("hex");
      state.openProcessedDatabase(makeProcessedDb());
      // openProcessedDatabase does NOT explicitly set viewMode
      expect(state.viewMode()).toBe("hex");
    });
  });

  // =========================================================================
  // Export tab
  // =========================================================================

  describe("openExportTab", () => {
    it("creates an export tab with fixed ID", () => {
      state.openExportTab();

      expect(state.tabs()).toHaveLength(1);
      const tab = state.tabs()[0];
      expect(tab.id).toBe("__export__");
      expect(tab.type).toBe("export");
      expect(tab.title).toBe("Export");
    });

    it("sets view mode to export", () => {
      state.openExportTab();
      expect(state.viewMode()).toBe("export");
    });

    it("reactivates if already open", () => {
      state.openExportTab();
      state.openEvidenceFile(makeFile());
      state.openExportTab();

      expect(state.tabs()).toHaveLength(2);
      expect(state.activeTabId()).toBe("__export__");
    });
  });

  // =========================================================================
  // Close tab
  // =========================================================================

  describe("closeTab", () => {
    it("removes the tab from the list", () => {
      state.openEvidenceFile(makeFile());
      expect(state.tabs()).toHaveLength(1);

      state.closeTab("evidence:/evidence/case.e01");
      expect(state.tabs()).toHaveLength(0);
    });

    it("tracks closed tab in recentlyClosed", () => {
      state.openEvidenceFile(makeFile());
      state.closeTab("evidence:/evidence/case.e01");

      expect(state.recentlyClosed().has("evidence:/evidence/case.e01")).toBe(true);
    });

    it("selects adjacent tab when closing active tab", () => {
      const file1 = makeFile({ path: "/a.e01", filename: "a.e01" });
      const file2 = makeFile({ path: "/b.e01", filename: "b.e01" });
      const file3 = makeFile({ path: "/c.e01", filename: "c.e01" });

      state.openEvidenceFile(file1);
      state.openEvidenceFile(file2);
      state.openEvidenceFile(file3);
      expect(state.activeTabId()).toBe("evidence:/c.e01");

      // Close the last tab — should select the new last tab
      state.closeTab("evidence:/c.e01");
      expect(state.activeTabId()).toBe("evidence:/b.e01");
    });

    it("selects first tab when closing first of multiple", () => {
      const file1 = makeFile({ path: "/a.e01", filename: "a.e01" });
      const file2 = makeFile({ path: "/b.e01", filename: "b.e01" });

      state.openEvidenceFile(file1);
      state.openEvidenceFile(file2);
      state.setActiveTabId("evidence:/a.e01"); // select first

      state.closeTab("evidence:/a.e01");
      expect(state.activeTabId()).toBe("evidence:/b.e01");
    });

    it("clears activeTabId when closing the only tab", () => {
      state.openEvidenceFile(makeFile());
      state.closeTab("evidence:/evidence/case.e01");
      expect(state.activeTabId()).toBeNull();
    });

    it("does not change activeTabId when closing inactive tab", () => {
      const file1 = makeFile({ path: "/a.e01", filename: "a.e01" });
      const file2 = makeFile({ path: "/b.e01", filename: "b.e01" });

      state.openEvidenceFile(file1);
      state.openEvidenceFile(file2);
      expect(state.activeTabId()).toBe("evidence:/b.e01");

      state.closeTab("evidence:/a.e01");
      expect(state.activeTabId()).toBe("evidence:/b.e01");
      expect(state.tabs()).toHaveLength(1);
    });

    it("is a no-op for unknown tab ID", () => {
      state.openEvidenceFile(makeFile());
      state.closeTab("nonexistent");
      expect(state.tabs()).toHaveLength(1);
    });
  });

  // =========================================================================
  // Close all tabs
  // =========================================================================

  describe("closeAllTabs", () => {
    it("removes all tabs", () => {
      state.openEvidenceFile(makeFile({ path: "/a.e01", filename: "a.e01" }));
      state.openEvidenceFile(makeFile({ path: "/b.e01", filename: "b.e01" }));
      state.openExportTab();

      state.closeAllTabs();
      expect(state.tabs()).toHaveLength(0);
      expect(state.activeTabId()).toBeNull();
    });

    it("tracks all closed tabs in recentlyClosed", () => {
      state.openEvidenceFile(makeFile({ path: "/a.e01", filename: "a.e01" }));
      state.openExportTab();

      state.closeAllTabs();
      expect(state.recentlyClosed().has("evidence:/a.e01")).toBe(true);
      expect(state.recentlyClosed().has("__export__")).toBe(true);
    });
  });

  // =========================================================================
  // Recently closed behavior
  // =========================================================================

  describe("recently closed tracking", () => {
    it("prevents re-opening a recently closed tab", () => {
      const file = makeFile();
      state.openEvidenceFile(file);
      state.closeTab("evidence:/evidence/case.e01");

      state.openEvidenceFile(file);
      expect(state.tabs()).toHaveLength(0);
    });

    it("clears recently closed when opening a different item", () => {
      const file1 = makeFile({ path: "/a.e01", filename: "a.e01" });
      const file2 = makeFile({ path: "/b.e01", filename: "b.e01" });

      state.openEvidenceFile(file1);
      state.closeTab("evidence:/a.e01");
      expect(state.recentlyClosed().size).toBe(1);

      // Opening a different file clears the recently closed set
      state.openEvidenceFile(file2);
      expect(state.recentlyClosed().size).toBe(0);
    });

    it("clearRecentlyClosed resets the set", () => {
      state.openEvidenceFile(makeFile());
      state.closeTab("evidence:/evidence/case.e01");
      expect(state.recentlyClosed().size).toBe(1);

      state.clearRecentlyClosed();
      expect(state.recentlyClosed().size).toBe(0);
    });

    it("allows re-opening after clearRecentlyClosed", () => {
      const file = makeFile();
      state.openEvidenceFile(file);
      state.closeTab("evidence:/evidence/case.e01");

      state.clearRecentlyClosed();
      state.openEvidenceFile(file);
      expect(state.tabs()).toHaveLength(1);
    });
  });

  // =========================================================================
  // Mixed tab types
  // =========================================================================

  describe("mixed tab types", () => {
    it("supports multiple tab types simultaneously", () => {
      state.openEvidenceFile(makeFile());
      state.openCaseDocument(makeCaseDoc());
      state.openContainerEntry(makeEntry());
      state.openProcessedDatabase(makeProcessedDb());
      state.openExportTab();

      expect(state.tabs()).toHaveLength(5);
      const types = state.tabs().map(t => t.type);
      expect(types).toContain("evidence");
      expect(types).toContain("document");
      expect(types).toContain("entry");
      expect(types).toContain("processed");
      expect(types).toContain("export");
    });

    it("last opened tab is active", () => {
      state.openEvidenceFile(makeFile());
      state.openExportTab();
      expect(state.activeTabType()).toBe("export");
    });
  });

  // =========================================================================
  // Derived state (activeTab, activeTabType)
  // =========================================================================

  describe("derived state", () => {
    it("activeTab returns the tab matching activeTabId", () => {
      const file = makeFile();
      state.openEvidenceFile(file);

      const tab = state.activeTab();
      expect(tab).not.toBeNull();
      expect(tab!.file).toBe(file);
    });

    it("activeTab returns null for unknown activeTabId", () => {
      state.setActiveTabId("nonexistent");
      expect(state.activeTab()).toBeNull();
    });

    it("activeTabType derives from active tab", () => {
      state.openExportTab();
      expect(state.activeTabType()).toBe("export");

      state.openEvidenceFile(makeFile());
      expect(state.activeTabType()).toBe("evidence");
    });
  });
});
