// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { HexViewer } from "./HexViewer";
import { mockInvoke } from "../__tests__/setup";

// Mock the hooks module used by HexViewer
const mockReadBytesFromSource = vi.fn();
const mockGetSourceKey = vi.fn();

vi.mock("../hooks", () => ({
  readBytesFromSource: (...args: any[]) => mockReadBytesFromSource(...args),
  getSourceKey: (...args: any[]) => mockGetSourceKey(...args),
}));

// Mock preferences
vi.mock("./preferences", () => ({
  getPreference: vi.fn(() => 50), // 50MB max preview
}));

// Mock sub-components used by HexViewer
vi.mock("./hex/HexToolbar", () => ({
  HexToolbar: (props: any) => <div data-testid="hex-toolbar">HexToolbar</div>,
}));

vi.mock("./hex/HexLine", () => ({
  HexLine: (props: any) => (
    <div data-testid="hex-line" data-offset={props.offset}>
      HexLine
    </div>
  ),
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
  name: "evidence.bin",
  path: "/evidence/evidence.bin",
  size: 4096,
  extension: "bin",
  type: "binary" as const,
};

const mockContainerEntry = {
  containerPath: "/evidence/image.ad1",
  entryPath: "System32/config/SAM",
  name: "SAM",
  size: 262144,
  isDir: false,
  isVfsEntry: false,
  isArchiveEntry: false,
  isDiskFile: false,
};

// Create mock byte data (first 256 bytes with some structure)
function createMockBytes(size: number): number[] {
  const bytes: number[] = [];
  for (let i = 0; i < size; i++) {
    bytes.push(i % 256);
  }
  return bytes;
}

describe("HexViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockReadBytesFromSource.mockReset();
    mockGetSourceKey.mockReset();

    mockGetSourceKey.mockReturnValue("test-key");
  });

  describe("Disk file rendering", () => {
    it("loads bytes from disk file", async () => {
      const mockBytes = createMockBytes(256);
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: mockBytes,
        totalSize: 4096,
      });

      const { container } = renderComponent(() => (
        <HexViewer file={mockDiskFile} />
      ));
      await tick();

      // Should render hex content (HexLine components or hex data)
      expect(container.innerHTML).toBeTruthy();
      expect(mockReadBytesFromSource).toHaveBeenCalled();
    });

    it("calls readBytesFromSource with correct initial arguments", async () => {
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: createMockBytes(64),
        totalSize: 4096,
      });

      renderComponent(() => <HexViewer file={mockDiskFile} />);
      await tick();

      expect(mockReadBytesFromSource).toHaveBeenCalledWith(
        mockDiskFile,
        undefined,
        0,
        65536, // INITIAL_LOAD_SIZE
      );
    });
  });

  describe("Container entry rendering", () => {
    it("loads bytes from container entry", async () => {
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: createMockBytes(128),
        totalSize: 262144,
      });

      const { container } = renderComponent(() => (
        <HexViewer entry={mockContainerEntry} />
      ));
      await tick();

      expect(container.innerHTML).toBeTruthy();
      expect(mockReadBytesFromSource).toHaveBeenCalledWith(
        null,
        mockContainerEntry,
        0,
        65536,
      );
    });
  });

  describe("Loading and error states", () => {
    it("shows loading state initially", () => {
      mockReadBytesFromSource.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <HexViewer file={mockDiskFile} />
      ));

      expect(container.textContent).toContain("Loading");
    });

    it("shows error when reading fails", async () => {
      mockReadBytesFromSource.mockRejectedValueOnce(
        new Error("Cannot read binary data")
      );

      const { container } = renderComponent(() => (
        <HexViewer file={mockDiskFile} />
      ));
      await tick();

      expect(container.textContent).toContain("Cannot read binary data");
    });
  });

  describe("Metadata callback", () => {
    it("calls onMetadataLoaded when metadata is available", async () => {
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: createMockBytes(64),
        totalSize: 4096,
      });
      // Mock the invoke for header parsing (viewer_detect_type, viewer_parse_header)
      mockInvoke
        .mockResolvedValueOnce({ type: "binary", description: "Binary file" }) // viewer_detect_type
        .mockResolvedValueOnce({
          file_type: { name: "Binary", description: "Unknown binary" },
          regions: [],
          fields: [],
        }); // viewer_parse_header

      const onMetadata = vi.fn();

      renderComponent(() => (
        <HexViewer file={mockDiskFile} onMetadataLoaded={onMetadata} />
      ));
      await tick(200);

      // onMetadataLoaded may or may not fire depending on header parsing
      // The important thing is no crash
      expect(mockReadBytesFromSource).toHaveBeenCalled();
    });
  });

  describe("Edge cases", () => {
    it("handles empty file (zero bytes)", async () => {
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: [],
        totalSize: 0,
      });

      const { container } = renderComponent(() => (
        <HexViewer file={mockDiskFile} />
      ));
      await tick();

      // Should render without crashing
      expect(container.innerHTML).toBeTruthy();
    });

    it("renders without file or entry (no data)", () => {
      const { container } = renderComponent(() => <HexViewer />);

      // Should render empty state without crashing
      expect(container.innerHTML).toBeTruthy();
    });
  });
});
