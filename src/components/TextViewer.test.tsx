// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { createSignal } from "solid-js";
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
        bytesRead: 33,
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
        bytesRead: 7,
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
        bytesRead: 22,
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

    it("ignores stale text loads after the selected file changes", async () => {
      let resolveSlow: (value: {
        text: string;
        bytesRead: number;
        totalSize: number;
      }) => void = () => {};
      const slowResult = new Promise<{
        text: string;
        bytesRead: number;
        totalSize: number;
      }>((resolve) => {
        resolveSlow = resolve;
      });
      const slowFile = {
        ...mockDiskFile,
        path: "/evidence/slow.txt",
        filename: "slow.txt",
      };
      const currentFile = {
        ...mockDiskFile,
        path: "/evidence/current.txt",
        filename: "current.txt",
      };

      mockReadTextFromSource.mockImplementation((file) => {
        if (file?.path === "/evidence/slow.txt") {
          return slowResult;
        }
        if (file?.path === "/evidence/current.txt") {
          return Promise.resolve({
            text: "current file content",
            bytesRead: 20,
            totalSize: 20,
          });
        }
        return Promise.reject(new Error("Unexpected file"));
      });

      const [file, setFile] = createSignal(slowFile);
      const { container } = renderComponent(() => <TextViewer file={file()} />);
      await tick();

      setFile(currentFile);
      await tick();

      expect(container.textContent).toContain("current file content");

      resolveSlow({
        text: "stale file content",
        bytesRead: 18,
        totalSize: 18,
      });
      await tick();

      expect(container.textContent).toContain("current file content");
      expect(container.textContent).not.toContain("stale file content");
    });
  });

  describe("Edge cases", () => {
    it("handles empty file", async () => {
      mockReadTextFromSource.mockResolvedValueOnce({
        text: "",
        bytesRead: 0,
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

  describe("incremental loading", () => {
    it("uses consumed bytes, not character count, as the next text offset", async () => {
      mockReadTextFromSource
        .mockResolvedValueOnce({
          text: "éé",
          bytesRead: 4,
          totalSize: 8,
        })
        .mockResolvedValueOnce({
          text: "ab",
          bytesRead: 2,
          totalSize: 8,
        });

      const { container } = renderComponent(() => (
        <TextViewer file={mockDiskFile} />
      ));
      await tick();

      const loadMoreButton = Array.from(container.querySelectorAll("button")).find(
        (button) => button.textContent?.includes("Load More"),
      ) as HTMLButtonElement;
      expect(loadMoreButton).toBeDefined();

      loadMoreButton.click();
      await tick();

      expect(mockReadTextFromSource).toHaveBeenNthCalledWith(
        2,
        mockDiskFile,
        undefined,
        4,
        4,
      );
      expect(container.textContent).toContain("ééab");
    });
  });
});
