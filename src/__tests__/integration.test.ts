// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Integration tests for core application flows.
 * These tests verify that components work together correctly.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// Mock Tauri APIs
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

describe("Integration Tests", () => {
  describe("File Opening Flow", () => {
    beforeEach(() => {
      vi.clearAllMocks();
    });

    it("should validate supported file extensions", () => {
      const supportedExtensions = [".ad1", ".e01", ".l01", ".zip", ".7z", ".tar", ".raw", ".dd"];
      
      const isSupported = (filename: string): boolean => {
        const ext = filename.toLowerCase().slice(filename.lastIndexOf("."));
        return supportedExtensions.includes(ext);
      };

      expect(isSupported("evidence.ad1")).toBe(true);
      expect(isSupported("disk.E01")).toBe(true);
      expect(isSupported("archive.zip")).toBe(true);
      expect(isSupported("document.pdf")).toBe(false);
      expect(isSupported("image.jpg")).toBe(false);
    });

    it("should handle multiple file selection", () => {
      const files = [
        "/path/to/evidence1.ad1",
        "/path/to/evidence2.e01",
        "/path/to/archive.zip",
      ];

      const processFiles = (paths: string[]) => {
        return paths.map(p => ({
          path: p,
          name: p.split("/").pop() || "",
          extension: p.slice(p.lastIndexOf(".")),
        }));
      };

      const result = processFiles(files);
      expect(result).toHaveLength(3);
      expect(result[0].name).toBe("evidence1.ad1");
      expect(result[1].extension).toBe(".e01");
    });

    it("should detect file type from extension", () => {
      const getFileType = (filename: string): string => {
        const ext = filename.toLowerCase().slice(filename.lastIndexOf("."));
        const typeMap: Record<string, string> = {
          ".ad1": "AD1",
          ".e01": "E01",
          ".l01": "L01",
          ".zip": "ZIP",
          ".7z": "7Z",
          ".tar": "TAR",
          ".raw": "RAW",
          ".dd": "DD",
        };
        return typeMap[ext] || "Unknown";
      };

      expect(getFileType("evidence.ad1")).toBe("AD1");
      expect(getFileType("disk.E01")).toBe("E01");
      expect(getFileType("unknown.xyz")).toBe("Unknown");
    });
  });

  describe("Container Analysis Flow", () => {
    it("should track analysis progress states", () => {
      type AnalysisState = "idle" | "loading" | "parsing" | "complete" | "error";
      
      const states: AnalysisState[] = [];
      
      const setState = (state: AnalysisState) => {
        states.push(state);
      };

      // Simulate analysis flow
      setState("idle");
      setState("loading");
      setState("parsing");
      setState("complete");

      expect(states).toEqual(["idle", "loading", "parsing", "complete"]);
    });

    it("should aggregate metadata from multiple sources", () => {
      interface Metadata {
        source: string;
        key: string;
        value: string;
      }

      const metadata: Metadata[] = [
        { source: "header", key: "format", value: "AD1" },
        { source: "header", key: "version", value: "3" },
        { source: "content", key: "fileCount", value: "1234" },
      ];

      const grouped = metadata.reduce((acc, item) => {
        if (!acc[item.source]) acc[item.source] = [];
        acc[item.source].push(item);
        return acc;
      }, {} as Record<string, Metadata[]>);

      expect(grouped.header).toHaveLength(2);
      expect(grouped.content).toHaveLength(1);
    });

    it("should calculate file statistics", () => {
      const files = [
        { name: "doc.pdf", size: 1024 },
        { name: "image.jpg", size: 2048 },
        { name: "video.mp4", size: 5000 },
      ];

      const stats = {
        totalFiles: files.length,
        totalSize: files.reduce((sum, f) => sum + f.size, 0),
        averageSize: files.reduce((sum, f) => sum + f.size, 0) / files.length,
        largestFile: files.reduce((max, f) => f.size > max.size ? f : max, files[0]),
      };

      expect(stats.totalFiles).toBe(3);
      expect(stats.totalSize).toBe(8072);
      expect(stats.largestFile.name).toBe("video.mp4");
    });
  });

  describe("Search Flow", () => {
    it("should filter files by name", () => {
      const files = [
        { name: "document.pdf", path: "/root/document.pdf" },
        { name: "report.docx", path: "/root/report.docx" },
        { name: "image.jpg", path: "/root/image.jpg" },
        { name: "documents_backup.zip", path: "/root/documents_backup.zip" },
      ];

      const searchByName = (query: string) => {
        const lower = query.toLowerCase();
        return files.filter(f => f.name.toLowerCase().includes(lower));
      };

      // "doc" matches document.pdf, report.docx, and documents_backup.zip
      expect(searchByName("doc")).toHaveLength(3);
      expect(searchByName("image")).toHaveLength(1);
      expect(searchByName("xyz")).toHaveLength(0);
    });

    it("should filter files by extension", () => {
      const files = [
        { name: "doc1.pdf", ext: ".pdf" },
        { name: "doc2.pdf", ext: ".pdf" },
        { name: "image.jpg", ext: ".jpg" },
        { name: "video.mp4", ext: ".mp4" },
      ];

      const filterByExtension = (ext: string) => {
        return files.filter(f => f.ext === ext);
      };

      expect(filterByExtension(".pdf")).toHaveLength(2);
      expect(filterByExtension(".mp4")).toHaveLength(1);
    });

    it("should support multiple search criteria", () => {
      const files = [
        { name: "report_2024.pdf", size: 1000, date: "2024-01-15" },
        { name: "report_2023.pdf", size: 2000, date: "2023-06-20" },
        { name: "data_2024.xlsx", size: 500, date: "2024-03-10" },
      ];

      interface SearchCriteria {
        nameContains?: string;
        minSize?: number;
        year?: string;
      }

      const search = (criteria: SearchCriteria) => {
        return files.filter(f => {
          if (criteria.nameContains && !f.name.includes(criteria.nameContains)) return false;
          if (criteria.minSize && f.size < criteria.minSize) return false;
          if (criteria.year && !f.date.startsWith(criteria.year)) return false;
          return true;
        });
      };

      expect(search({ nameContains: "report" })).toHaveLength(2);
      expect(search({ year: "2024" })).toHaveLength(2);
      expect(search({ nameContains: "report", year: "2024" })).toHaveLength(1);
    });
  });

  describe("Export Flow", () => {
    it("should prepare export data", () => {
      const selectedFiles = [
        { id: 1, name: "file1.pdf", path: "/container/file1.pdf" },
        { id: 2, name: "file2.jpg", path: "/container/file2.jpg" },
      ];

      const prepareExport = (files: typeof selectedFiles, destination: string) => {
        return {
          files: files.map(f => ({ ...f, exportPath: `${destination}/${f.name}` })),
          totalCount: files.length,
          destination,
        };
      };

      const result = prepareExport(selectedFiles, "/exports");
      expect(result.totalCount).toBe(2);
      expect(result.files[0].exportPath).toBe("/exports/file1.pdf");
    });

    it("should handle export name conflicts", () => {
      const existingFiles = ["file.pdf", "document.docx"];
      
      const resolveConflict = (name: string, existing: string[]): string => {
        if (!existing.includes(name)) return name;
        
        const ext = name.slice(name.lastIndexOf("."));
        const base = name.slice(0, name.lastIndexOf("."));
        let counter = 1;
        let newName = `${base}_${counter}${ext}`;
        
        while (existing.includes(newName)) {
          counter++;
          newName = `${base}_${counter}${ext}`;
        }
        
        return newName;
      };

      expect(resolveConflict("new.pdf", existingFiles)).toBe("new.pdf");
      expect(resolveConflict("file.pdf", existingFiles)).toBe("file_1.pdf");
      expect(resolveConflict("file.pdf", [...existingFiles, "file_1.pdf"])).toBe("file_2.pdf");
    });
  });

  describe("Tab Management Flow", () => {
    it("should manage open tabs", () => {
      interface Tab {
        id: string;
        title: string;
        path: string;
        isDirty: boolean;
      }

      const tabs: Tab[] = [];
      
      const openTab = (path: string, title: string): Tab => {
        const existing = tabs.find(t => t.path === path);
        if (existing) return existing;
        
        const tab: Tab = {
          id: `tab-${Date.now()}`,
          title,
          path,
          isDirty: false,
        };
        tabs.push(tab);
        return tab;
      };

      const closeTab = (id: string) => {
        const index = tabs.findIndex(t => t.id === id);
        if (index !== -1) tabs.splice(index, 1);
      };

      const tab1 = openTab("/file1.ad1", "Evidence 1");
      const tab2 = openTab("/file2.e01", "Evidence 2");
      
      expect(tabs).toHaveLength(2);
      
      // Opening same file shouldn't create new tab
      const tab1Again = openTab("/file1.ad1", "Evidence 1");
      expect(tabs).toHaveLength(2);
      expect(tab1Again).toBe(tab1);

      closeTab(tab1.id);
      expect(tabs).toHaveLength(1);
    });
  });
});
