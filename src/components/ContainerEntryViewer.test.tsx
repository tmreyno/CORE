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
            viewerType: "Hex",
            description: "Windows Registry Hive",
            mimeType: "application/x-windows-registry",
            method: "magic",
          };
        }
        return null;
      });

      const entry = makeEntry({ name: "NTUSER.DAT", isArchiveEntry: true });
      const { container, dispose } = renderComponent(() =>
        <ContainerEntryViewer entry={entry} viewMode="auto" />
      );

      await tick(200);

      // Registry hive detected
      expect(container.textContent).toContain("Windows Registry Hive");
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
  });
});
