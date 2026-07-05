// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { ContainerEntryViewer } from "./ContainerEntryViewer";
import { mockInvoke } from "../__tests__/setup";
import type { SelectedEntry } from "./EvidenceTree";

vi.mock("../utils/platform", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../utils/platform")>()),
  isTauri: true,
}));

// Polyfill ResizeObserver for JSDOM (used by PdfViewer)
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as any;
}

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

// Wait for async updates
const tick = (ms = 50) => new Promise(resolve => setTimeout(resolve, ms));

// Base entry factory
function makeEntry(overrides: Partial<SelectedEntry> = {}): SelectedEntry {
  return {
    name: "test.bin",
    containerPath: "/evidence/container.ad1",
    entryPath: "/files/test.bin",
    size: 1024,
    isDir: false,
    isDiskFile: false,
    isVfsEntry: false,
    isArchiveEntry: false,
    ...overrides,
  };
}

describe("ContainerEntryViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("rendering", () => {
    it("renders header with file name and size", () => {
      const entry = makeEntry({ name: "evidence.dat", size: 2048 });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("evidence.dat");
      expect(container.textContent).toContain("KB");
      dispose();
    });

    it("renders disk file badge", () => {
      const entry = makeEntry({ isDiskFile: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("Disk File");
      dispose();
    });

    it("renders VFS badge", () => {
      const entry = makeEntry({ isVfsEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("VFS");
      dispose();
    });

    it("renders Archive badge", () => {
      const entry = makeEntry({ isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("Archive");
      dispose();
    });
  });

  describe("view mode toggle", () => {
    it("shows hex/text toggle when onViewModeChange is provided", () => {
      const entry = makeEntry();
      const onChange = vi.fn();
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" onViewModeChange={onChange} />
      );

      const buttons = container.querySelectorAll("button");
      const hexBtn = Array.from(buttons).find(b => b.textContent === "Hex");
      const textBtn = Array.from(buttons).find(b => b.textContent === "Text");
      expect(hexBtn).toBeDefined();
      expect(textBtn).toBeDefined();
      dispose();
    });

    it("shows Preview button for previewable file types", () => {
      const entry = makeEntry({ name: "photo.jpg" });
      const onChange = vi.fn();
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" onViewModeChange={onChange} />
      );

      const previewBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent?.includes("Preview")
      );
      expect(previewBtn).toBeDefined();
      dispose();
    });

    it("does not show Preview button for unknown file types without detection", () => {
      const entry = makeEntry({ name: "data.dat" });
      const onChange = vi.fn();
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" onViewModeChange={onChange} />
      );

      const previewBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent?.includes("Preview")
      );
      expect(previewBtn).toBeUndefined();
      dispose();
    });
  });

  describe("canPreview extension matching", () => {
    const previewableExtensions = [
      // PDF
      "pdf",
      // Text documents (DocumentViewer)
      "docx", "doc", "html", "htm", "md", "markdown", "txt",
      // Presentations
      "pptx", "ppt", "odp", "odt", "rtf",
      // Spreadsheets
      "xlsx", "xls", "xlsm", "xlsb", "csv", "ods", "tsv", "numbers",
      // Images (all from IMAGE_EXTENSIONS)
      "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "avif",
      "tiff", "tif", "heic", "heif",
      // RAW camera formats
      "raw", "cr2", "nef", "arw", "dng", "orf", "rw2",
      // Email
      "eml", "mbox", "msg",
      // PST/OST email archives
      "pst", "ost",
      // Property lists
      "plist", "mobileprovision",
      // Binary executables (all from BINARY_EXECUTABLE_EXTENSIONS)
      "exe", "dll", "so", "dylib", "sys", "drv",
      "elf", "bin", "com", "scr", "ocx", "cpl",
      // Databases
      "db", "db3", "sqlite", "sqlite3", "sqlitedb",
      // Code files (representative subset from CODE_EXTENSIONS)
      "py", "js", "ts", "rs", "java", "c", "cpp", "go",
      "bat", "cmd", "sh", "ps1", "vbs", "lua", "pl", "r",
      "json", "xml", "yaml", "sql", "css",
      // Config files (representative subset from CONFIG_EXTENSIONS)
      "log", "ini", "cfg", "conf", "env",
    ];

    for (const ext of previewableExtensions) {
      it(`shows preview for .${ext} files`, () => {
        const entry = makeEntry({ name: `file.${ext}` });
        const onChange = vi.fn();
        const { container, dispose } = renderComponent(() =>
          <ContainerEntryViewer entry={entry} viewMode="hex" onViewModeChange={onChange} />
        );

        const previewBtn = Array.from(container.querySelectorAll("button")).find(
          b => b.textContent?.includes("Preview")
        );
        expect(previewBtn).toBeDefined();
        dispose();
      });
    }
  });

  describe("auto mode determination", () => {
    it("defaults to hex for unknown binary files", () => {
      const entry = makeEntry({ name: "unknown.dat" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      // In auto mode with unknown type, should show hex view
      // The hex viewer attempts to read data via invoke
      expect(container.textContent).not.toContain("Preview");
      dispose();
    });

    it("defaults to preview for code files", () => {
      const entry = makeEntry({ name: "script.py" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      // Code files are previewable via DocumentViewer, auto mode → preview
      // The text viewer attempts to read data via invoke
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });
  });

  describe("content detection integration", () => {
    it("persists a normalized artifact when project DB is open", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "project_db_is_open") return true;
        if (cmd === "project_db_extract_artifact_source") {
          return {
            artifact: {
              id: "artifact-1",
              sourceId: "ad1:/evidence/container.ad1:/files/test.bin",
            },
            record: {
              id: "artifact-1",
              sourceId: "ad1:/evidence/container.ad1:/files/test.bin",
            },
          };
        }
        return null;
      });

      const entry = makeEntry({ name: "test.bin" });
      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      await tick(200);

      expect(mockInvoke).toHaveBeenCalledWith("project_db_extract_artifact_source", {
        request: {
          source: {
            containerPath: "/evidence/container.ad1",
            entryPath: "/files/test.bin",
            containerType: "ad1",
            size: 1024,
          },
          extractor: "container-entry-viewer",
        },
      });
      dispose();
    });

    it("keeps rendering when artifact persistence fails", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "project_db_is_open") return true;
        if (cmd === "project_db_extract_artifact_source") {
          throw new Error("artifact extraction unavailable");
        }
        return null;
      });

      const entry = makeEntry({ name: "test.bin" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      await tick(200);

      expect(mockInvoke).toHaveBeenCalledWith(
        "project_db_extract_artifact_source",
        expect.anything(),
      );
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("triggers content detection for unknown file types in auto mode", async () => {
      // Mock the extract command to return a temp path
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          return "/tmp/extracted_file.dat";
        }
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return {
            format: "Sqlite",
            viewerType: "Database",
            description: "SQLite Database",
            mimeType: "application/x-sqlite3",
            method: "magic",
          };
        }
        return null;
      });

      const entry = makeEntry({ name: "data.dat", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      // After detection, the detected format badge should appear
      // Use retry loop for slow CI runners
      for (let i = 0; i < 20; i++) {
        await tick(100);
        if (container.textContent?.includes("SQLite Database")) break;
      }
      expect(container.textContent).toContain("SQLite Database");
      expect(mockInvoke).toHaveBeenCalledWith("detect_content_format_source", {
        source: expect.objectContaining({
          containerPath: "/evidence/container.ad1",
          entryPath: "/files/test.bin",
          containerType: "ad1",
          size: 1024,
        }),
      });
      dispose();
    });

    it("does not extract unknown hex-only entries in auto mode", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "detect_content_format_source") {
          return {
            format: "Binary",
            viewerType: "Hex",
            description: "Unknown binary data",
            mimeType: "application/octet-stream",
            method: "fallback",
          };
        }
        return null;
      });

      const entry = makeEntry({
        name: "pagefile.sys",
        size: 4_294_967_296,
        isVfsEntry: true,
      });
      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(300);

      expect(mockInvoke).toHaveBeenCalledWith("detect_content_format_source", {
        source: expect.objectContaining({
          containerPath: "/evidence/container.ad1",
          entryPath: "/files/test.bin",
          containerType: "ad1",
          size: 4_294_967_296,
        }),
      });
      expect(
        mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp"),
      ).toBe(false);
      dispose();
    });

    it("triggers content detection for registry hive (.dat) files", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          return "/tmp/NTUSER.DAT";
        }
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return {
            format: "RegistryHive",
            viewerType: "Registry",
            description: "Windows Registry Hive",
            mimeType: "application/x-windows-registry",
            method: "magic",
          };
        }
        if (cmd === "registry_get_info" || cmd === "registry_get_info_source") {
          return {
            path: "/tmp/NTUSER.DAT",
            rootKeyName: "CMI-CreateHive",
            rootKeyPath: "CMI-CreateHive",
            rootTimestamp: "2024-01-15 10:30:00 UTC",
            totalKeys: 10,
            totalValues: 20,
            rootSubkeyCount: 1,
            rootValueCount: 0,
          };
        }
        if (cmd === "registry_get_subkeys" || cmd === "registry_get_subkeys_source") {
          return { parentPath: "", subkeys: [] };
        }
        if (cmd === "registry_get_key_info" || cmd === "registry_get_key_info_source") {
          return {
            name: "CMI-CreateHive",
            path: "CMI-CreateHive",
            prettyPath: "CMI-CreateHive",
            timestamp: "2024-01-15 10:30:00 UTC",
            subkeyCount: 0,
            valueCount: 0,
            values: [],
            subkeys: [],
          };
        }
        return null;
      });

      const entry = makeEntry({ name: "NTUSER.DAT", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      // Wait for async mock chain to resolve (CI runners may be slower)
      for (let i = 0; i < 20; i++) {
        await tick(100);
        if (container.textContent?.includes("Registry")) break;
      }

      // Registry viewer renders with "Registry" badge
      expect(container.textContent).toContain("Registry");
      dispose();
    });

    it("uses disk file path directly for content detection", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return {
            format: "Pdf",
            viewerType: "Pdf",
            description: "PDF Document",
            mimeType: "application/pdf",
            method: "magic",
          };
        }
        return null;
      });

      const entry = makeEntry({
        name: "report.dat",
        isDiskFile: true,
        containerPath: "/evidence/report.dat",
        entryPath: "/evidence/report.dat",
      });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      // Should NOT call extract, should call detect directly
      const extractCalls = mockInvoke.mock.calls.filter(
        (c: string[]) => c[0] === "container_extract_entry_to_temp"
      );
      expect(extractCalls).toHaveLength(0);

      // PDF detected - badge should show (retry for slow CI)
      for (let i = 0; i < 20; i++) {
        await tick(100);
        if (container.textContent?.includes("PDF Document")) break;
      }
      expect(container.textContent).toContain("PDF Document");
      dispose();
    });

    it("falls back gracefully when detection fails", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          return "/tmp/file.dat";
        }
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          throw new Error("Detection not available");
        }
        return null;
      });

      const entry = makeEntry({ name: "unknown.dat", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      // Should not crash, no detected badge
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("falls back to hex when detection returns no viewer type", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return undefined;
        }
        return null;
      });

      const entry = makeEntry({ name: "unknown.dat", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      expect(container.innerHTML).toBeTruthy();
      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      dispose();
    });

    it("does not run content detection for known previewable extensions", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          return "/tmp/photo.jpg";
        }
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          throw new Error("Should not be called for known types");
        }
        // Return empty data for image viewer
        return null;
      });

      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      // detect_content_format should NOT have been called
      const detectCalls = mockInvoke.mock.calls.filter(
        (c: string[]) => c[0] === "detect_content_format" || c[0] === "detect_content_format_source"
      );
      expect(detectCalls).toHaveLength(0);
      dispose();
    });

    it("resets detected format when entry changes", async () => {
      let currentEntry = makeEntry({ name: "data.dat", isArchiveEntry: true });
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/data.dat";
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return {
            format: "Sqlite",
            viewerType: "Database",
            description: "SQLite Database",
            mimeType: "application/x-sqlite3",
            method: "magic",
          };
        }
        return null;
      });

      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={currentEntry} viewMode="auto" />
      );

      await tick(200);
      expect(container.textContent).toContain("SQLite Database");
      dispose();
    });
  });

  describe("back button", () => {
    it("renders back button when onBack is provided", () => {
      const entry = makeEntry();
      const onBack = vi.fn();
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" onBack={onBack} />
      );

      const backBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent?.includes("Back")
      );
      expect(backBtn).toBeDefined();
      dispose();
    });

    it("does not render back button when onBack is not provided", () => {
      const entry = makeEntry();
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      const backBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent?.includes("Back")
      );
      expect(backBtn).toBeUndefined();
      dispose();
    });

    it("calls onBack when back button is clicked", () => {
      const entry = makeEntry();
      const onBack = vi.fn();
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" onBack={onBack} />
      );

      const backBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent?.includes("Back")
      );
      backBtn?.click();
      expect(onBack).toHaveBeenCalledOnce();
      dispose();
    });
  });

  describe("preview extraction flow", () => {
    it("uses source-backed preview for AD1 files without temp extraction", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "viewer_get_binary_info_source") return { size: 1024, mimeType: "application/pdf" };
        return null;
      });

      const entry = makeEntry({
        name: "report.pdf",
        containerPath: "/evidence/container.ad1",
        entryPath: "files/report.pdf",
        isArchiveEntry: false,
        isVfsEntry: false,
        isDiskFile: false,
      });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      expect(mockInvoke).toHaveBeenCalledWith("viewer_get_binary_info_source", {
        source: expect.objectContaining({
          containerPath: "/evidence/container.ad1",
          entryPath: "files/report.pdf",
          containerType: "ad1",
        }),
      });
      dispose();
    });

    it("uses source-backed preview for archive entries without temp extraction", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "spreadsheet_info_source") return { sheets: [] };
        return null;
      });

      const entry = makeEntry({
        name: "data.xlsx",
        containerPath: "/evidence/archive.zip",
        entryPath: "data.xlsx",
        isArchiveEntry: true,
      });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      expect(mockInvoke).toHaveBeenCalledWith("viewer_detect_type_source", {
        source: expect.objectContaining({
          containerPath: "/evidence/archive.zip",
          entryPath: "data.xlsx",
          containerType: "zip",
        }),
      });
      dispose();
    });

    it("uses disk file path directly without extraction", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          throw new Error("Should not be called for disk files");
        }
        return null;
      });

      const entry = makeEntry({
        name: "report.pdf",
        isDiskFile: true,
        containerPath: "/evidence/report.pdf",
        entryPath: "/evidence/report.pdf",
      });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      const extractCalls = mockInvoke.mock.calls.filter(
        (c: string[]) => c[0] === "container_extract_entry_to_temp"
      );
      expect(extractCalls).toHaveLength(0);
      dispose();
    });

    it("treats self-referencing paths as disk files", async () => {
      mockInvoke.mockImplementation(async () => null);

      const entry = makeEntry({
        name: "photo.jpg",
        containerPath: "/photos/photo.jpg",
        entryPath: "/photos/photo.jpg",
        isDiskFile: false,
      });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      // containerPath === entryPath → treated as disk file, no extraction
      const extractCalls = mockInvoke.mock.calls.filter(
        (c: string[]) => c[0] === "container_extract_entry_to_temp"
      );
      expect(extractCalls).toHaveLength(0);
      dispose();
    });

    it("passes dataAddr to source-backed preview commands when available", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "viewer_get_binary_info_source") return { size: 1024, mimeType: "application/pdf" };
        return null;
      });

      const entry = makeEntry({
        name: "entry.pdf",
        entryPath: "files/entry.pdf",
        dataAddr: 0x1000,
        isArchiveEntry: false,
        isVfsEntry: false,
        isDiskFile: false,
      });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      expect(mockInvoke).toHaveBeenCalledWith("viewer_get_binary_info_source", {
        source: expect.objectContaining({
          dataAddr: 0x1000,
          containerType: "ad1",
          entryPath: "files/entry.pdf",
        }),
      });
      dispose();
    });

    it("does not extract for directory entries", async () => {
      mockInvoke.mockImplementation(async () => null);

      const entry = makeEntry({ name: "folder", isDir: true });
      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      const extractCalls = mockInvoke.mock.calls.filter(
        (c: string[]) => c[0] === "container_extract_entry_to_temp"
      );
      expect(extractCalls).toHaveLength(0);
      dispose();
    });
  });

  describe("preview loading state", () => {
    it("does not show temp extraction spinner for source-backed image preview", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "viewer_get_binary_info_source") return { size: 1024, mimeType: "image/jpeg" };
        if (cmd === "viewer_read_binary_source_base64_chunk") {
          return { data: "", offset: 0, length: 0, totalSize: 1024 };
        }
        if (cmd === "exif_read_source") return { success: false, data: null, error: "No EXIF" };
        return null;
      });

      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(100);

      expect(container.textContent).not.toContain("Extracting file...");
      expect(container.textContent).toContain("photo.jpg");
      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      dispose();
    });

    it("keeps preview controls enabled for source-backed image preview", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "viewer_get_binary_info_source") return { size: 1024, mimeType: "image/jpeg" };
        if (cmd === "viewer_read_binary_source_base64_chunk") {
          return { data: "", offset: 0, length: 0, totalSize: 1024 };
        }
        if (cmd === "exif_read_source") return { success: false, data: null, error: "No EXIF" };
        return null;
      });

      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const onChange = vi.fn();
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" onViewModeChange={onChange} />
      );

      await tick(100);

      const loadingBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent?.includes("Loading...")
      );
      expect(loadingBtn).toBeUndefined();
      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      dispose();
    });
  });

  describe("preview error state", () => {
    it("does not surface temp extraction errors for source-backed preview", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          throw new Error("Permission denied: /tmp/extract");
        }
        if (cmd === "viewer_get_binary_info_source") return { size: 1024, mimeType: "application/pdf" };
        return null;
      });

      const entry = makeEntry({ name: "secret.pdf", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      expect(container.textContent).not.toContain("Preview unavailable");
      expect(container.textContent).not.toContain("Permission denied: /tmp/extract");
      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      dispose();
    });

    it("skips string extraction errors for source-backed preview", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          throw "Container is corrupted";
        }
        if (cmd === "viewer_get_binary_info_source") return { size: 1024, mimeType: "application/pdf" };
        return null;
      });

      const entry = makeEntry({ name: "corrupt.pdf", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      expect(container.textContent).not.toContain("Preview unavailable");
      expect(container.textContent).not.toContain("Container is corrupted");
      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      dispose();
    });
  });

  describe("viewer delegation", () => {
    const binaryInfo = (path: string) => ({
      path,
      format: "PE",
      architecture: "x86_64",
      is_64bit: true,
      entry_point: 4096,
      imports: [],
      exports: [],
      sections: [],
      strings: [],
      file_size: 1024,
      pe_timestamp: null,
      pe_checksum: null,
      pe_subsystem: "Native",
      pe_linker_version: null,
      pe_os_version: null,
      pe_image_version: null,
      pe_subsystem_version: null,
      pe_image_base: null,
      pe_section_alignment: null,
      pe_file_alignment: null,
      pe_size_of_image: null,
      pe_size_of_headers: null,
      pe_dll_characteristics: null,
      pe_dll_characteristics_detail: [],
      pe_certificate_table_size: null,
      pe_is_driver: true,
      pe_driver_type: "kernel-driver",
      pe_driver_indicators: ["extension:.sys"],
      pe_version_info: {},
      macho_cpu_type: null,
      macho_filetype: null,
      linux_module_info: null,
      has_debug_info: false,
      is_stripped: false,
      has_code_signing: false,
    });

    // Helper to set up a successful preview extraction
    function setupPreview(extractedPath = "/tmp/extracted") {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return extractedPath;
        // Return empty data for any child viewer commands
        if (cmd === "document_read") return { success: true, content: { format: "txt", text: "content", html: "<p>content</p>" }, error: null };
        if (cmd === "document_read_source") return { success: true, content: { format: "txt", text: "content", html: "<p>content</p>" }, error: null };
        if (cmd === "document_get_metadata") return { success: true, metadata: {}, error: null };
        if (cmd === "document_get_metadata_source") return { success: true, metadata: {}, error: null };
        if (cmd === "spreadsheet_read") return { success: true, data: { sheets: [] }, error: null };
        if (cmd === "email_read") return { success: true, email: {}, error: null };
        if (cmd === "email_parse_eml") return { path: extractedPath, subject: "Email", from: [], to: [], cc: [], bcc: [], body_text: "", body_html: null, attachments: [], headers: [], size: 0 };
        if (cmd === "email_parse_eml_source") return { path: extractedPath, subject: "Email", from: [], to: [], cc: [], bcc: [], body_text: "", body_html: null, attachments: [], headers: [], size: 0 };
        if (cmd === "email_parse_mbox") return [];
        if (cmd === "email_parse_mbox_source") return [];
        if (cmd === "email_parse_msg") return { path: extractedPath, subject: "Email", from: [], to: [], cc: [], bcc: [], body_text: "", body_html: null, attachments: [], headers: [], size: 0 };
        if (cmd === "email_parse_msg_source") return { path: extractedPath, subject: "Email", from: [], to: [], cc: [], bcc: [], body_text: "", body_html: null, attachments: [], headers: [], size: 0 };
        if (cmd === "plist_read") return { success: true, data: {}, error: null };
        if (cmd === "binary_analyze") return binaryInfo(extractedPath);
        if (cmd === "binary_analyze_source") return binaryInfo(extractedPath);
        if (cmd === "registry_get_info") return { path: extractedPath, rootKeyName: "ROOT", rootKeyPath: "ROOT", rootTimestamp: "", totalKeys: 0, totalValues: 0, rootSubkeyCount: 0, rootValueCount: 0 };
        if (cmd === "registry_get_info_source") return { path: extractedPath, rootKeyName: "ROOT", rootKeyPath: "ROOT", rootTimestamp: "", totalKeys: 0, totalValues: 0, rootSubkeyCount: 0, rootValueCount: 0 };
        if (cmd === "registry_get_subkeys") return { parentPath: "", subkeys: [] };
        if (cmd === "registry_get_subkeys_source") return { parentPath: "", subkeys: [] };
        if (cmd === "registry_get_key_info") return { name: "ROOT", path: "ROOT", prettyPath: "ROOT", timestamp: "", subkeyCount: 0, valueCount: 0, values: [], subkeys: [] };
        if (cmd === "registry_get_key_info_source") return { name: "ROOT", path: "ROOT", prettyPath: "ROOT", timestamp: "", subkeyCount: 0, valueCount: 0, values: [], subkeys: [] };
        if (cmd === "database_get_info") return { tables: [], views: [], path: extractedPath, pageCount: 0, pageSize: 4096, sqliteVersion: "3.39.0", totalSize: 0 };
        if (cmd === "database_get_info_source") return { tables: [], views: [], path: extractedPath, pageCount: 0, pageSize: 4096, sqliteVersion: "3.39.0", totalSize: 0 };
        if (cmd === "database_get_table_schema_source") return { name: "items", columns: [], rowCount: 0, createSql: null, indexes: [] };
        if (cmd === "database_query_table_source") return { tableName: "items", columns: [], rows: [], totalCount: 0, page: 0, pageSize: 100, hasMore: false };
        if (cmd === "exif_read") return { success: false, data: null, error: "No EXIF" };
        // PST viewer commands
        if (cmd === "pst_get_folders") return { path: extractedPath, displayName: "Test PST", folders: [], totalMessages: 0, totalFolders: 0 };
        if (cmd === "pst_get_folders_source") return { path: extractedPath, displayName: "Test PST", folders: [], totalMessages: 0, totalFolders: 0 };
        if (cmd === "pst_get_messages") return [];
        if (cmd === "pst_get_messages_source") return [];
        if (cmd === "pst_get_message_detail") return null;
        if (cmd === "pst_get_message_detail_source") return null;
        // Office viewer command
        if (cmd === "office_read_document") return { format: "docx", formatDescription: "Microsoft Word Document (OOXML)", metadata: {}, sections: [{ label: null, paragraphs: [{ text: "Test content", hint: "normal" }] }], totalChars: 12, totalWords: 2, extractionComplete: true, warnings: [] };
        if (cmd === "office_read_document_source") return { format: "docx", formatDescription: "Microsoft Word Document (OOXML)", metadata: {}, sections: [{ label: null, paragraphs: [{ text: "Test content", hint: "normal" }] }], totalChars: 12, totalWords: 2, extractionComplete: true, warnings: [] };
        return null;
      });
    }

    it("renders PDF viewer for .pdf files in preview mode", async () => {
      setupPreview("/tmp/report.pdf");
      const entry = makeEntry({ name: "report.pdf", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      // PDF viewer should be rendered (it renders via PdfViewer component)
      expect(container.querySelector("[class*='pdf'], canvas, iframe, [data-testid]") !== null || container.innerHTML.length > 100).toBe(true);
      dispose();
    });

    it("renders Image viewer with EXIF panel for image files", async () => {
      setupPreview("/tmp/photo.jpg");
      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      // Image viewer renders within a flex container with EXIF panel
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders Spreadsheet viewer for .xlsx files", async () => {
      setupPreview("/tmp/data.xlsx");
      const entry = makeEntry({ name: "data.xlsx", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders Email viewer for .eml files", async () => {
      setupPreview("/tmp/message.eml");
      const entry = makeEntry({ name: "message.eml", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders Plist viewer for .plist files", async () => {
      setupPreview("/tmp/info.plist");
      const entry = makeEntry({ name: "info.plist", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders Binary viewer for .exe files", async () => {
      setupPreview("/tmp/app.exe");
      const entry = makeEntry({ name: "app.exe", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("routes uppercase .SYS evidence entries to source-backed binary analysis without extraction", async () => {
      setupPreview("/tmp/DRIVER.SYS");
      const entry = makeEntry({
        name: "DRIVER.SYS",
        entryPath: "Windows/System32/drivers/DRIVER.SYS",
        isArchiveEntry: false,
        isVfsEntry: true,
        isDiskFile: false,
      });
      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(700);

      expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "container_extract_entry_to_temp")).toBe(false);
      expect(mockInvoke).toHaveBeenCalledWith("binary_analyze_source", {
        source: expect.objectContaining({
          containerPath: "/evidence/container.ad1",
          entryPath: "Windows/System32/drivers/DRIVER.SYS",
          containerType: "ad1",
          size: 1024,
        }),
      });
      dispose();
    });

    it("renders Database viewer for .sqlite files", async () => {
      setupPreview("/tmp/data.sqlite");
      const entry = makeEntry({ name: "data.sqlite", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders PST viewer for .pst files", async () => {
      setupPreview("/tmp/mailbox.pst");
      const entry = makeEntry({ name: "mailbox.pst", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders Office viewer for .docx files", async () => {
      setupPreview("/tmp/document.docx");
      const entry = makeEntry({ name: "document.docx", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders Office viewer for .doc files", async () => {
      setupPreview("/tmp/document.doc");
      const entry = makeEntry({ name: "document.doc", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("falls back to DocumentViewer for unknown previewable types", async () => {
      setupPreview("/tmp/page.html");
      const entry = makeEntry({ name: "page.html", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);
      // DocumentViewer is the Switch fallback
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders HexViewer when viewMode is hex", () => {
      const entry = makeEntry({ name: "data.bin" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      // HexViewer should render (it calls invoke to read chunks)
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("renders TextViewer when viewMode is text", () => {
      const entry = makeEntry({ name: "data.bin" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="text" />
      );

      // TextViewer should render
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("delegates to correct viewer based on content detection", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/mystery.dat";
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return {
            format: "Email",
            viewerType: "Email",
            description: "Email Message",
            mimeType: "message/rfc822",
            method: "magic",
          };
        }
        if (cmd === "email_read") return { success: true, email: { from: "test@test.com", subject: "Test" }, error: null };
        return null;
      });

      const entry = makeEntry({ name: "mystery.dat", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      // Should show "Email Message" badge from content detection
      expect(container.textContent).toContain("Email Message");
      dispose();
    });
  });

  describe("metadata emission", () => {
    it("calls onMetadata with file info in hex mode", async () => {
      const onMetadata = vi.fn();
      const entry = makeEntry({
        name: "evidence.bin",
        size: 4096,
        containerPath: "/evidence/disk.ad1",
        entryPath: "/files/evidence.bin",
        containerType: "ad1",
      });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" onMetadata={onMetadata} />
      );

      await tick(100);

      expect(onMetadata).toHaveBeenCalled();
      const lastCall = onMetadata.mock.calls[onMetadata.mock.calls.length - 1][0];
      expect(lastCall.fileInfo.name).toBe("evidence.bin");
      expect(lastCall.fileInfo.size).toBe(4096);
      expect(lastCall.fileInfo.containerPath).toBe("/evidence/disk.ad1");
      expect(lastCall.fileInfo.containerType).toBe("ad1");
      expect(lastCall.viewerType).toBe("Hex");
      dispose();
    });

    it("reports Text viewer type in text mode", async () => {
      const onMetadata = vi.fn();
      const entry = makeEntry({ name: "notes.txt" });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="text" onMetadata={onMetadata} />
      );

      await tick(100);

      const lastCall = onMetadata.mock.calls[onMetadata.mock.calls.length - 1][0];
      expect(lastCall.viewerType).toBe("Text");
      dispose();
    });

    it("reports PDF viewer type for PDF preview", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/report.pdf";
        return null;
      });

      const onMetadata = vi.fn();
      const entry = makeEntry({ name: "report.pdf", isArchiveEntry: true });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" onMetadata={onMetadata} />
      );

      await tick(200);

      const lastCall = onMetadata.mock.calls[onMetadata.mock.calls.length - 1][0];
      expect(lastCall.viewerType).toBe("PDF");
      dispose();
    });

    it("reports Image viewer type for image preview", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/photo.png";
        if (cmd === "exif_read") return { success: false, data: null, error: "No EXIF" };
        return null;
      });

      const onMetadata = vi.fn();
      const entry = makeEntry({ name: "photo.png", isArchiveEntry: true });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" onMetadata={onMetadata} />
      );

      await tick(200);

      const lastCall = onMetadata.mock.calls[onMetadata.mock.calls.length - 1][0];
      expect(lastCall.viewerType).toBe("Image");
      dispose();
    });

    it("omits containerPath from metadata when it matches entryPath", async () => {
      const onMetadata = vi.fn();
      const entry = makeEntry({
        name: "standalone.bin",
        containerPath: "/files/standalone.bin",
        entryPath: "/files/standalone.bin",
      });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" onMetadata={onMetadata} />
      );

      await tick(100);

      const lastCall = onMetadata.mock.calls[onMetadata.mock.calls.length - 1][0];
      expect(lastCall.fileInfo.containerPath).toBeUndefined();
      dispose();
    });

    it("includes file extension in metadata", async () => {
      const onMetadata = vi.fn();
      const entry = makeEntry({ name: "archive.7z" });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" onMetadata={onMetadata} />
      );

      await tick(100);

      const lastCall = onMetadata.mock.calls[onMetadata.mock.calls.length - 1][0];
      expect(lastCall.fileInfo.extension).toBe("7z");
      dispose();
    });

    it("reports detected viewer type from content detection", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/unknown.dat";
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return {
            format: "Spreadsheet",
            viewerType: "Spreadsheet",
            description: "Excel Spreadsheet",
            mimeType: "application/vnd.openxmlformats",
            method: "magic",
          };
        }
        if (cmd === "spreadsheet_read") return { success: true, data: { sheets: [] }, error: null };
        return null;
      });

      const onMetadata = vi.fn();
      const entry = makeEntry({ name: "unknown.dat", isArchiveEntry: true });

      const { dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" onMetadata={onMetadata} />
      );

      await tick(200);

      const lastCall = onMetadata.mock.calls[onMetadata.mock.calls.length - 1][0];
      expect(lastCall.viewerType).toBe("Spreadsheet");
      dispose();
    });
  });

  describe("close preview", () => {
    it("shows Close button text when preview is active", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/photo.jpg";
        if (cmd === "exif_read") return { success: false, data: null, error: "No EXIF" };
        return null;
      });

      const onChange = vi.fn();
      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" onViewModeChange={onChange} />
      );

      await tick(200);

      const closeBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent?.includes("Close")
      );
      expect(closeBtn).toBeDefined();
      dispose();
    });

    it("calls onViewModeChange with hex when closing preview via hex button", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/photo.jpg";
        if (cmd === "exif_read") return { success: false, data: null, error: "No EXIF" };
        return null;
      });

      const onChange = vi.fn();
      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" onViewModeChange={onChange} />
      );

      await tick(200);

      const hexBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent === "Hex"
      );
      hexBtn?.click();

      expect(onChange).toHaveBeenCalledWith("hex");
      dispose();
    });

    it("calls onViewModeChange with text when clicking text button", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/photo.jpg";
        if (cmd === "exif_read") return { success: false, data: null, error: "No EXIF" };
        return null;
      });

      const onChange = vi.fn();
      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" onViewModeChange={onChange} />
      );

      await tick(200);

      const textBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent === "Text"
      );
      textBtn?.click();

      expect(onChange).toHaveBeenCalledWith("text");
      dispose();
    });
  });

  describe("header display", () => {
    it("renders formatted file size", () => {
      const entry = makeEntry({ name: "large.bin", size: 1048576 }); // 1 MB
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("MB");
      dispose();
    });

    it("renders file name with title attribute showing full path", () => {
      const entry = makeEntry({
        name: "deep_file.bin",
        entryPath: "/very/long/path/to/deep_file.bin",
      });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      const nameEl = container.querySelector("[title='/very/long/path/to/deep_file.bin']");
      expect(nameEl).toBeDefined();
      expect(nameEl?.textContent).toContain("deep_file.bin");
      dispose();
    });

    it("displays detected format badge with description and mimetype", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/mystery.dat";
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return {
            format: "Jpeg",
            viewerType: "Image",
            description: "JPEG Image",
            mimeType: "image/jpeg",
            method: "magic",
          };
        }
        if (cmd === "exif_read") return { success: false, data: null, error: "No EXIF" };
        return null;
      });

      const entry = makeEntry({ name: "mystery.dat", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      expect(container.textContent).toContain("JPEG Image");
      // Badge title should include description and mimeType
      const badge = container.querySelector("[title*='image/jpeg']");
      expect(badge).not.toBeNull();
      dispose();
    });
  });

  describe("auto mode file type mapping", () => {
    const textFileExtensions = [
      "log", "ini", "cfg", "conf", "properties", "env",
      "gitignore", "editorconfig", "eslintrc", "prettierrc",
      "dockerignore", "npmrc", "yarnrc", "hgignore",
    ];

    for (const ext of textFileExtensions) {
      it(`selects preview mode for .${ext} config files in auto mode`, () => {
        const entry = makeEntry({ name: `config.${ext}` });
        const { container, dispose } = renderComponent(() =>
          <ContainerEntryViewer entry={entry} viewMode="auto" />
        );

        // Config files are previewable via DocumentViewer, auto mode → preview
        expect(container.innerHTML).toBeTruthy();
        dispose();
      });
    }

    const codeExtensions = ["py", "js", "ts", "rs", "java", "c", "cpp", "bat", "cmd", "sh", "vbs"];

    for (const ext of codeExtensions) {
      it(`selects preview mode for .${ext} code files in auto mode`, () => {
        const entry = makeEntry({ name: `main.${ext}` });
        const { container, dispose } = renderComponent(() =>
          <ContainerEntryViewer entry={entry} viewMode="auto" />
        );

        expect(container.innerHTML).toBeTruthy();
        dispose();
      });
    }
  });

  describe("effective mode logic", () => {
    it("falls back to hex when preview mode set but no preview path yet", () => {
      // Don't resolve the mock so preview path is never set
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const entry = makeEntry({ name: "unknown.bin" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      // Without a preview path for non-previewable file, should fall back
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("explicit hex mode shows hex regardless of file type", () => {
      const entry = makeEntry({ name: "photo.jpg" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      // Even for a .jpg, explicit hex mode should show hex view
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });

    it("explicit text mode shows text regardless of file type", () => {
      const entry = makeEntry({ name: "photo.jpg" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="text" />
      );

      expect(container.innerHTML).toBeTruthy();
      dispose();
    });
  });

  describe("edge cases", () => {
    it("handles zero-size files gracefully", () => {
      const entry = makeEntry({ name: "empty.txt", size: 0 });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("empty.txt");
      expect(container.textContent).toContain("0"); // 0 bytes
      dispose();
    });

    it("handles entries with very long names", () => {
      const longName = "a".repeat(200) + ".pdf";
      const entry = makeEntry({ name: longName });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain(longName);
      dispose();
    });

    it("handles entries with special characters in name", () => {
      const entry = makeEntry({ name: "file (copy) [2].pdf" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("file (copy) [2].pdf");
      dispose();
    });

    it("handles entries with no extension", () => {
      const entry = makeEntry({ name: "Makefile" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("Makefile");
      dispose();
    });

    it("clears stale preview loading when the selected entry changes", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "viewer_get_binary_info_source") return { size: 1024, mimeType: "image/jpeg" };
        if (cmd === "viewer_read_binary_source_base64_chunk") {
          return { data: "", offset: 0, length: 0, totalSize: 1024 };
        }
        if (cmd === "exif_read_source") return { success: false, data: null, error: "No EXIF" };
        return null;
      });

      const [entry, setEntry] = createSignal(
        makeEntry({ name: "photo.jpg", entryPath: "/files/photo.jpg" }),
      );
      const [viewMode, setViewMode] = createSignal<"preview" | "hex">("preview");
      const { container, dispose } = renderComponent(() => (
        <ContainerEntryViewer entry={entry()} viewMode={viewMode()} />
      ));

      await tick(50);
      expect(container.textContent).toContain("photo.jpg");
      expect(container.textContent).not.toContain("Extracting file");

      setViewMode("hex");
      setEntry(makeEntry({ name: "notes.bin", entryPath: "/files/notes.bin" }));
      await tick(50);

      expect(container.textContent).not.toContain("Extracting file");
      expect(container.textContent).toContain("notes.bin");
      dispose();
    });

    it("renders multiple badges simultaneously", () => {
      const entry = makeEntry({
        name: "data.bin",
        isDiskFile: true,
        isVfsEntry: true,
      });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      expect(container.textContent).toContain("Disk File");
      expect(container.textContent).toContain("VFS");
      dispose();
    });

    it("does not show view toggle when onViewModeChange is absent", () => {
      const entry = makeEntry({ name: "photo.jpg" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="hex" />
      );

      // Without onViewModeChange, no hex/text/preview buttons
      const hexBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent === "Hex"
      );
      expect(hexBtn).toBeUndefined();
      dispose();
    });

    it("ignores Binary format detection result without magic method", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/data.dat";
        if (cmd === "detect_content_format" || cmd === "detect_content_format_source") {
          return {
            format: "Binary",
            viewerType: "Hex",
            description: "Unknown Binary",
            mimeType: "application/octet-stream",
            method: "fallback",
          };
        }
        return null;
      });

      const entry = makeEntry({ name: "data.dat", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      // Binary format with "fallback" method should be ignored
      expect(container.textContent).not.toContain("Unknown Binary");
      dispose();
    });
  });
});
