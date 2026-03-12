// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createSignal } from "solid-js";

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../utils/logger", () => ({
  logger: {
    scope: () => ({
      debug: vi.fn(),
      warn: vi.fn(),
      info: vi.fn(),
      error: vi.fn(),
    }),
  },
}));

vi.mock("../../utils/accessibility", () => ({
  announce: vi.fn(),
}));

vi.mock("../../api/search", () => ({
  searchQuery: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

import { searchQuery } from "../../api/search";
const mockSearchQuery = vi.mocked(searchQuery);

import { createSearchHandlers, createContextMenuBuilders } from "../useAppActions";
import type { DiscoveredFile } from "../../types";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const makeFile = (path: string, containerType = "ad1"): DiscoveredFile => ({
  path,
  filename: path.split("/").pop() || path,
  size: 1024,
  container_type: containerType,
});

const mockFileManager = (files: DiscoveredFile[] = []) => {
  const [activeFile, setActiveFile] = createSignal<DiscoveredFile | null>(null);
  const [selectedFiles] = createSignal(new Set<string>());

  return {
    discoveredFiles: () => files,
    activeFile,
    setActiveFile,
    scanDir: () => "/evidence",
    selectedFiles,
    toggleFileSelection: vi.fn(),
    setScanDir: vi.fn(),
    scanForFiles: vi.fn(),
    setOk: vi.fn(),
  };
};

const mockHashManager = () => ({
  hashSingleFile: vi.fn(),
  hashSelectedFiles: vi.fn(),
  hashAllFiles: vi.fn(),
  selectedHashAlgorithm: () => "SHA-256",
});

const mockProjectManager = () => ({
  projectPath: () => "/case/project.cffx",
  autoSaveEnabled: () => false as boolean,
  setAutoSaveEnabled: vi.fn(),
  modified: () => false as boolean,
  saveProject: vi.fn().mockResolvedValue({ success: true }),
  saveProjectAs: vi.fn().mockResolvedValue({ success: true }),
  project: () => null as any,
});

const mockToast = () => ({
  success: vi.fn(),
  error: vi.fn(),
  info: vi.fn(),
});

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

beforeEach(() => {
  vi.clearAllMocks();
  // Default: Tantivy not available → triggers in-memory fallback
  mockSearchQuery.mockRejectedValue(new Error("No index available"));
});

// ---------------------------------------------------------------------------
// createSearchHandlers
// ---------------------------------------------------------------------------

describe("createSearchHandlers", () => {
  describe("handleSearch", () => {
    it("returns empty results for no matching files", async () => {
      const fm = mockFileManager([makeFile("/evidence/disk.e01")]);
      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      const results = await handleSearch("nonexistent", {} as any);
      expect(results).toEqual([]);
    });

    it("matches files by name", async () => {
      const fm = mockFileManager([
        makeFile("/evidence/disk.e01"),
        makeFile("/evidence/phone.ufd"),
      ]);
      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      const results = await handleSearch("disk", {} as any);
      expect(results).toHaveLength(1);
      expect(results[0].name).toBe("disk.e01");
      expect(results[0].score).toBe(100);
      expect(results[0].matchType).toBe("name");
    });

    it("matches files by path with lower score", async () => {
      const fm = mockFileManager([makeFile("/evidence/cases/disk.e01")]);
      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      const results = await handleSearch("cases", {} as any);
      expect(results).toHaveLength(1);
      expect(results[0].score).toBe(50);
      expect(results[0].matchType).toBe("path");
    });

    it("is case-insensitive", async () => {
      const fm = mockFileManager([makeFile("/evidence/DISK.E01")]);
      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      const results = await handleSearch("disk", {} as any);
      expect(results).toHaveLength(1);
    });

    it("searches containers via Tantivy for queries >= 2 chars", async () => {
      const fm = mockFileManager([makeFile("/evidence/archive.zip", "zip")]);
      mockSearchQuery.mockResolvedValueOnce({
        hits: [
          {
            docId: "/evidence/archive.zip:/inner/secret.txt",
            containerPath: "/evidence/archive.zip",
            containerType: "zip",
            entryPath: "/inner/secret.txt",
            filename: "secret.txt",
            extension: "txt",
            size: 256,
            modified: 0,
            isDir: false,
            fileCategory: "document",
            score: 0.8,
            snippet: "",
            contentMatch: false,
          },
        ],
        totalHits: 1,
        elapsedMs: 5,
        categoryCounts: [],
        containerTypeCounts: [],
      });

      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });
      const results = await handleSearch("secret", {} as any);

      expect(mockSearchQuery).toHaveBeenCalledWith(expect.objectContaining({ query: "secret" }));
      const containerResult = results.find((r) => r.containerPath);
      expect(containerResult).toBeDefined();
      expect(containerResult!.name).toBe("secret.txt");
    });

    it("does not search containers for single-char queries", async () => {
      const fm = mockFileManager([makeFile("/evidence/a.zip", "zip")]);
      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      await handleSearch("a", {} as any);
      expect(mockInvoke).not.toHaveBeenCalled();
    });

    it("handles Tantivy search errors gracefully", async () => {
      const fm = mockFileManager([makeFile("/evidence/archive.zip", "zip")]);
      // searchQuery rejects by default (from beforeEach) — falls back to in-memory

      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });
      const results = await handleSearch("test", {} as any);

      // Should still return file-level results (empty in this case since "test" doesn't match "archive.zip")
      expect(results).toEqual([]);
    });

    it("sorts results by score descending", async () => {
      const fm = mockFileManager([
        makeFile("/evidence/path/to/target.e01"),
        makeFile("/evidence/target.ad1"),
      ]);
      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      const results = await handleSearch("target", {} as any);
      expect(results.length).toBeGreaterThanOrEqual(2);
      // Name match scores higher than path match
      for (let i = 1; i < results.length; i++) {
        expect(results[i - 1].score).toBeGreaterThanOrEqual(results[i].score);
      }
    });

    it("limits results to 300", async () => {
      const files = Array.from({ length: 400 }, (_, i) =>
        makeFile(`/evidence/match_${i}.e01`)
      );
      const fm = mockFileManager(files);
      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      const results = await handleSearch("match", {} as any);
      expect(results.length).toBeLessThanOrEqual(300);
    });

    it("includes FTS cross-entity results when project is loaded", async () => {
      const fm = mockFileManager([]);
      const pm = mockProjectManager();
      (pm as any).project = () => ({ name: "Test Project" });
      
      // First call: rebuild_fts (resolves), second: fts_search (returns results)
      mockInvoke
        .mockResolvedValueOnce(undefined) // rebuild_fts
        .mockResolvedValueOnce([          // fts_search
          { source: "bookmarks", id: "bm-1", snippet: "important <mark>evidence</mark>", rank: -5.0 },
          { source: "notes", id: "note-1", snippet: "case <mark>evidence</mark> found", rank: -3.0 },
        ]);

      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: pm as any });
      const results = await handleSearch("evidence", {} as any);

      const ftsResults = results.filter((r) => r.id.startsWith("fts:"));
      expect(ftsResults).toHaveLength(2);
      expect(ftsResults[0].matchType).toBe("bookmarks");
      expect(ftsResults[1].matchType).toBe("notes");
    });

    it("handles FTS search errors gracefully", async () => {
      const fm = mockFileManager([]);
      const pm = mockProjectManager();
      (pm as any).project = () => ({ name: "Test Project" });
      
      mockInvoke
        .mockResolvedValueOnce(undefined)              // rebuild_fts
        .mockRejectedValueOnce(new Error("fts error")); // fts_search fails

      const { handleSearch } = createSearchHandlers({ fileManager: fm as any, projectManager: pm as any });
      const results = await handleSearch("test", {} as any);

      // Should still return empty (no file matches), no crash
      expect(results).toEqual([]);
    });
  });

  describe("handleSearchResultSelect", () => {
    it("sets active file for top-level file results", () => {
      const file = makeFile("/evidence/disk.e01");
      const fm = mockFileManager([file]);
      const { handleSearchResultSelect } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      handleSearchResultSelect({
        id: file.path,
        path: file.path,
        name: "disk.e01",
        size: 1024,
        isDir: false,
        score: 100,
        matchType: "name",
      });

      expect(fm.activeFile()).toBe(file);
    });

    it("sets active file to container for container results", () => {
      const container = makeFile("/evidence/archive.zip");
      const fm = mockFileManager([container]);
      const { handleSearchResultSelect } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      handleSearchResultSelect({
        id: "/evidence/archive.zip::/inner/file.txt",
        path: "/inner/file.txt",
        name: "file.txt",
        size: 256,
        isDir: false,
        score: 80,
        matchType: "name",
        containerPath: "/evidence/archive.zip",
      });

      expect(fm.activeFile()).toBe(container);
    });

    it("handles FTS cross-entity results without navigation crash", () => {
      const fm = mockFileManager([]);
      const { handleSearchResultSelect } = createSearchHandlers({ fileManager: fm as any, projectManager: mockProjectManager() as any });

      // FTS result should not throw
      handleSearchResultSelect({
        id: "fts:bookmarks:bm-1",
        path: "bm-1",
        name: "[bookmarks] important evidence",
        size: 0,
        isDir: false,
        score: 80,
        matchType: "bookmarks",
      });

      // Should not set active file
      expect(fm.activeFile()).toBeNull();
    });
  });
});

// ---------------------------------------------------------------------------
// createContextMenuBuilders
// ---------------------------------------------------------------------------

describe("createContextMenuBuilders", () => {
  describe("getFileContextMenuItems", () => {
    it("returns empty array when file is null", () => {
      const fm = mockFileManager();
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      const { getFileContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getFileContextMenuItems(() => null);
      expect(items).toEqual([]);
    });

    it("returns context menu items for a file", () => {
      const file = makeFile("/evidence/disk.e01");
      const fm = mockFileManager([file]);
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      const { getFileContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getFileContextMenuItems(() => file);

      // Should have: Open, sep, Hash, sep, Select, sep, Copy Path, Copy Name
      expect(items.length).toBeGreaterThanOrEqual(5);
      expect(items.find((i) => i.id === "open")).toBeDefined();
      expect(items.find((i) => i.id === "hash")).toBeDefined();
      expect(items.find((i) => i.id === "copy-path")).toBeDefined();
      expect(items.find((i) => i.id === "copy-name")).toBeDefined();
    });

    it("open action sets active file", () => {
      const file = makeFile("/evidence/disk.e01");
      const fm = mockFileManager([file]);
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      const { getFileContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getFileContextMenuItems(() => file);
      items.find((i) => i.id === "open")!.onSelect!();
      expect(fm.activeFile()).toBe(file);
    });

    it("hash action calls hashSingleFile", () => {
      const file = makeFile("/evidence/disk.e01");
      const fm = mockFileManager([file]);
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      const { getFileContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getFileContextMenuItems(() => file);
      items.find((i) => i.id === "hash")!.onSelect!();
      expect(hm.hashSingleFile).toHaveBeenCalledWith(file);
    });

    it("copy-path copies file path to clipboard", async () => {
      const file = makeFile("/evidence/disk.e01");
      const fm = mockFileManager([file]);
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      // Mock clipboard
      const writeText = vi.fn().mockResolvedValue(undefined);
      Object.assign(navigator, { clipboard: { writeText } });

      const { getFileContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getFileContextMenuItems(() => file);
      items.find((i) => i.id === "copy-path")!.onSelect!();
      expect(writeText).toHaveBeenCalledWith("/evidence/disk.e01");
    });

    it("shows 'Deselect' for already-selected files", () => {
      const file = makeFile("/evidence/disk.e01");
      const fm = mockFileManager([file]);
      // Override selectedFiles to include this file
      (fm as any).selectedFiles = () => new Set([file.path]);
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      const { getFileContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getFileContextMenuItems(() => file);
      const selectItem = items.find((i) => i.id === "select");
      expect(selectItem!.label).toBe("Deselect");
    });
  });

  describe("getSaveContextMenuItems", () => {
    it("returns save menu items", () => {
      const fm = mockFileManager();
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      const { getSaveContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getSaveContextMenuItems();
      expect(items.find((i) => i.id === "auto-save")).toBeDefined();
      expect(items.find((i) => i.id === "save")).toBeDefined();
      expect(items.find((i) => i.id === "save-as")).toBeDefined();
    });

    it("disables save when no evidence loaded", () => {
      const fm = mockFileManager();
      (fm as any).scanDir = () => null;
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      const { getSaveContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getSaveContextMenuItems();
      const saveItem = items.find((i) => i.id === "save");
      expect(saveItem!.disabled).toBe(true);
    });

    it("disables save when no existing project", () => {
      const fm = mockFileManager();
      const hm = mockHashManager();
      const pm = mockProjectManager();
      (pm as any).projectPath = () => null;
      const toast = mockToast();

      const { getSaveContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getSaveContextMenuItems();
      const saveItem = items.find((i) => i.id === "save");
      expect(saveItem!.disabled).toBe(true);
    });

    it("shows auto-save checked state", () => {
      const fm = mockFileManager();
      const hm = mockHashManager();
      const pm = mockProjectManager();
      (pm as any).autoSaveEnabled = () => true;
      const toast = mockToast();

      const { getSaveContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getSaveContextMenuItems();
      const autoSave = items.find((i) => i.id === "auto-save");
      expect(autoSave!.checked).toBe(true);
    });

    it("auto-save toggle calls setAutoSaveEnabled", () => {
      const fm = mockFileManager();
      const hm = mockHashManager();
      const pm = mockProjectManager();
      const toast = mockToast();

      const { getSaveContextMenuItems } = createContextMenuBuilders({
        fileManager: fm as any,
        hashManager: hm as any,
        projectManager: pm as any,
        toast,
        buildSaveOptions: () => null,
      });

      const items = getSaveContextMenuItems();
      items.find((i) => i.id === "auto-save")!.onSelect!();
      expect(pm.setAutoSaveEnabled).toHaveBeenCalledWith(true);
    });
  });
});
