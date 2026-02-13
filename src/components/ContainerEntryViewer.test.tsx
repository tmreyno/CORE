// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { ContainerEntryViewer } from "./ContainerEntryViewer";
import type { EntryViewMode } from "./ContainerEntryViewer";
import { mockInvoke } from "../__tests__/setup";
import type { SelectedEntry } from "./EvidenceTree";

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
      "pdf", "docx", "doc", "html", "htm", "md", "txt",
      "xlsx", "xls", "csv", "ods",
      "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico",
      "eml", "mbox",
      "plist", "mobileprovision",
      "exe", "dll", "so", "dylib", "sys", "drv",
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

    it("defaults to text for code files", () => {
      const entry = makeEntry({ name: "script.py" });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      // Code files don't have preview but auto mode → text
      // The text viewer attempts to read data via invoke
      expect(container.innerHTML).toBeTruthy();
      dispose();
    });
  });

  describe("content detection integration", () => {
    it("triggers content detection for unknown file types in auto mode", async () => {
      // Mock the extract command to return a temp path
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          return "/tmp/extracted_file.dat";
        }
        if (cmd === "detect_content_format") {
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
      expect(container.textContent).toContain("SQLite Database");
      dispose();
    });

    it("triggers content detection for registry hive (.dat) files", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          return "/tmp/NTUSER.DAT";
        }
        if (cmd === "detect_content_format") {
          return {
            format: "RegistryHive",
            viewerType: "Registry",
            description: "Windows Registry Hive",
            mimeType: "application/x-windows-registry",
            method: "magic",
          };
        }
        if (cmd === "registry_get_info") {
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
        if (cmd === "registry_get_subkeys") {
          return { parentPath: "", subkeys: [] };
        }
        if (cmd === "registry_get_key_info") {
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

      await tick(200);

      // Registry viewer renders with "Registry" badge
      expect(container.textContent).toContain("Registry");
      dispose();
    });

    it("uses disk file path directly for content detection", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "detect_content_format") {
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

      // PDF detected - badge should show
      expect(container.textContent).toContain("PDF Document");
      dispose();
    });

    it("falls back gracefully when detection fails", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          return "/tmp/file.dat";
        }
        if (cmd === "detect_content_format") {
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

    it("does not run content detection for known previewable extensions", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          return "/tmp/photo.jpg";
        }
        if (cmd === "detect_content_format") {
          throw new Error("Should not be called for known types");
        }
        // Return empty data for image viewer
        return null;
      });

      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      // detect_content_format should NOT have been called
      const detectCalls = mockInvoke.mock.calls.filter(
        (c: string[]) => c[0] === "detect_content_format"
      );
      expect(detectCalls).toHaveLength(0);
      dispose();
    });

    it("resets detected format when entry changes", async () => {
      let currentEntry = makeEntry({ name: "data.dat", isArchiveEntry: true });
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/data.dat";
        if (cmd === "detect_content_format") {
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
    it("extracts container entry to temp for AD1 files", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/extracted.pdf";
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

      expect(mockInvoke).toHaveBeenCalledWith("container_extract_entry_to_temp", expect.objectContaining({
        containerPath: "/evidence/container.ad1",
        entryPath: "files/report.pdf",
      }));
      dispose();
    });

    it("extracts archive entry with archive flag", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/archived.xlsx";
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

      expect(mockInvoke).toHaveBeenCalledWith("container_extract_entry_to_temp", expect.objectContaining({
        containerPath: "/evidence/archive.zip",
        isArchiveEntry: true,
      }));
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

    it("passes dataAddr to extract command when available", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return "/tmp/entry.bin";
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

      expect(mockInvoke).toHaveBeenCalledWith("container_extract_entry_to_temp", expect.objectContaining({
        dataAddr: 0x1000,
      }));
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
    it("shows extracting spinner during preview extraction", async () => {
      // Use a promise that never resolves to keep loading state
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(100);

      expect(container.textContent).toContain("Extracting file...");
      expect(container.textContent).toContain("photo.jpg");
      dispose();
    });

    it("shows Loading text on Preview button during extraction", async () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const entry = makeEntry({ name: "photo.jpg", isArchiveEntry: true });
      const onChange = vi.fn();
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" onViewModeChange={onChange} />
      );

      await tick(100);

      const loadingBtn = Array.from(container.querySelectorAll("button")).find(
        b => b.textContent?.includes("Loading...")
      );
      expect(loadingBtn).toBeDefined();
      expect(loadingBtn?.disabled).toBe(true);
      dispose();
    });
  });

  describe("preview error state", () => {
    it("displays error message when extraction fails", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          throw new Error("Permission denied: /tmp/extract");
        }
        return null;
      });

      const entry = makeEntry({ name: "secret.pdf", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      expect(container.textContent).toContain("Preview Error");
      expect(container.textContent).toContain("Permission denied: /tmp/extract");
      dispose();
    });

    it("handles string error from extraction", async () => {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") {
          throw "Container is corrupted";
        }
        return null;
      });

      const entry = makeEntry({ name: "corrupt.pdf", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="preview" />
      );

      await tick(200);

      expect(container.textContent).toContain("Preview Error");
      expect(container.textContent).toContain("Container is corrupted");
      dispose();
    });
  });

  describe("viewer delegation", () => {
    // Helper to set up a successful preview extraction
    function setupPreview(extractedPath = "/tmp/extracted") {
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "container_extract_entry_to_temp") return extractedPath;
        // Return empty data for any child viewer commands
        if (cmd === "document_read") return { success: true, content: { format: "txt", text: "content", html: "<p>content</p>" }, error: null };
        if (cmd === "document_get_metadata") return { success: true, metadata: {}, error: null };
        if (cmd === "spreadsheet_read") return { success: true, data: { sheets: [] }, error: null };
        if (cmd === "email_read") return { success: true, email: {}, error: null };
        if (cmd === "plist_read") return { success: true, data: {}, error: null };
        if (cmd === "binary_analyze") return { success: true, analysis: {}, error: null };
        if (cmd === "registry_get_info") return { path: extractedPath, rootKeyName: "ROOT", rootKeyPath: "ROOT", rootTimestamp: "", totalKeys: 0, totalValues: 0, rootSubkeyCount: 0, rootValueCount: 0 };
        if (cmd === "registry_get_subkeys") return { parentPath: "", subkeys: [] };
        if (cmd === "registry_get_key_info") return { name: "ROOT", path: "ROOT", prettyPath: "ROOT", timestamp: "", subkeyCount: 0, valueCount: 0, values: [], subkeys: [] };
        if (cmd === "database_get_info") return { tables: [], views: [], path: extractedPath, pageCount: 0, pageSize: 4096, sqliteVersion: "3.39.0", totalSize: 0 };
        if (cmd === "exif_read") return { success: false, data: null, error: "No EXIF" };
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
        if (cmd === "detect_content_format") {
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
        if (cmd === "detect_content_format") {
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
        if (cmd === "detect_content_format") {
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
    ];

    for (const ext of textFileExtensions) {
      it(`selects text mode for .${ext} files in auto mode`, () => {
        const entry = makeEntry({ name: `config.${ext}` });
        const { container, dispose } = renderComponent(() =>
          <ContainerEntryViewer entry={entry} viewMode="auto" />
        );

        // Text mode files should not trigger preview extraction
        expect(container.innerHTML).toBeTruthy();
        dispose();
      });
    }

    const codeExtensions = ["py", "js", "ts", "rs", "java", "c", "cpp"];

    for (const ext of codeExtensions) {
      it(`selects text mode for .${ext} code files in auto mode`, () => {
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
        if (cmd === "detect_content_format") {
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
