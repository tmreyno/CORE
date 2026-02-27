// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { TextViewer } from "./TextViewer";

// Mock the hooks module used by TextViewer
const mockReadTextFromSource = vi.fn();
const mockGetSourceKey = vi.fn();
const mockGetSourceFilename = vi.fn();

vi.mock("../hooks", () => ({
  readTextFromSource: (...args: any[]) => mockReadTextFromSource(...args),
  getSourceKey: (...args: any[]) => mockGetSourceKey(...args),
  getSourceFilename: (...args: any[]) => mockGetSourceFilename(...args),
}));

// Mock preferences
vi.mock("./preferences", () => ({
  getPreference: vi.fn(() => 50), // 50MB max preview
}));

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

// Wait for async updates
const tick = (ms = 50) => new Promise(resolve => setTimeout(resolve, ms));

// Mock file objects
const mockDiskFile = {
  filename: "readme.txt",
  path: "/evidence/readme.txt",
  size: 1024,
  container_type: "raw",
};

const mockContainerEntry = {
  containerPath: "/evidence/image.ad1",
  entryPath: "Documents/notes.txt",
  name: "notes.txt",
  size: 512,
  isDir: false,
  isVfsEntry: false,
  isArchiveEntry: false,
  isDiskFile: false,
};

describe("TextViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockReadTextFromSource.mockReset();
    mockGetSourceKey.mockReset();
    mockGetSourceFilename.mockReset();

    // Default mock returns
    mockGetSourceKey.mockReturnValue("test-key");
    mockGetSourceFilename.mockReturnValue("test.txt");
  });

  describe("Disk file rendering", () => {
    it("loads text content from disk file", async () => {
      mockReadTextFromSource.mockResolvedValueOnce({
        text: "Hello, World!\nThis is a test file.",
        totalSize: 33,
      });

      const { container } = renderComponent(() => (
        <TextViewer file={mockDiskFile} />
      ));
      await tick();

      expect(container.textContent).toContain("Hello, World!");
      expect(container.textContent).toContain("This is a test file.");
    });

    it("calls readTextFromSource with correct arguments", async () => {
      mockReadTextFromSource.mockResolvedValueOnce({
        text: "content",
        totalSize: 7,
      });

      renderComponent(() => <TextViewer file={mockDiskFile} />);
      await tick();

      expect(mockReadTextFromSource).toHaveBeenCalledWith(
        mockDiskFile,
        undefined,
        0,
        100000, // INITIAL_LOAD_SIZE
      );
    });
  });

  describe("Container entry rendering", () => {
    it("loads text from container entry", async () => {
      mockReadTextFromSource.mockResolvedValueOnce({
        text: "Container file content",
        totalSize: 22,
      });

      const { container } = renderComponent(() => (
        <TextViewer entry={mockContainerEntry} />
      ));
      await tick();

      expect(container.textContent).toContain("Container file content");
    });
  });

  describe("Loading and error states", () => {
    it("shows loading state initially", () => {
      mockReadTextFromSource.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <TextViewer file={mockDiskFile} />
      ));

      expect(container.textContent).toContain("Loading");
    });

    it("shows error when reading fails", async () => {
      mockReadTextFromSource.mockRejectedValueOnce(
        new Error("Failed to read file")
      );

      const { container } = renderComponent(() => (
        <TextViewer file={mockDiskFile} />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to read file");
    });
  });

  describe("Edge cases", () => {
    it("handles empty file", async () => {
      mockReadTextFromSource.mockResolvedValueOnce({
        text: "",
        totalSize: 0,
      });

      const { container } = renderComponent(() => (
        <TextViewer file={mockDiskFile} />
      ));
      await tick();

      // Should render without crashing
      expect(container.innerHTML).toBeTruthy();
    });

    it("renders without file or entry (no data)", () => {
      const { container } = renderComponent(() => <TextViewer />);

      // Should render empty state without crashing
      expect(container.innerHTML).toBeTruthy();
    });
  });
});
