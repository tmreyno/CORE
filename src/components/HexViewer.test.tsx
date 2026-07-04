// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render } from "solid-js/web";
import { HexViewer } from "./HexViewer";
import { mockInvoke } from "../__tests__/setup";

vi.mock("../utils/platform", () => ({
  isTauri: true,
}));

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
  HexToolbar: () => <div data-testid="hex-toolbar">HexToolbar</div>,
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
  filename: "evidence.bin",
  path: "/evidence/evidence.bin",
  size: 4096,
  container_type: "raw",
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

const mockSourceAnalysis = {
  sourceRef: { kind: "localFile", path: "/evidence/evidence.bin" },
  sourceId: "/evidence/evidence.bin",
  totalSize: 4096,
  offset: 0,
  bytesAnalyzed: 64,
  magicHex: "25 50 44 46 2D",
  signatures: [
    {
      offset: 0,
      description: "PDF Document",
      mimeType: "application/pdf",
      extensions: ["pdf"],
      category: "document",
      confidence: "high",
      magicHex: "25 50 44 46 2D",
    },
    {
      offset: 128,
      description: "PNG Image",
      mimeType: "image/png",
      extensions: ["png"],
      category: "image",
      confidence: "high",
      magicHex: "89 50 4E 47 0D 0A 1A 0A",
    },
  ],
  entropy: 4.25,
  entropyWindows: [{ offset: 0, length: 64, entropy: 4.25 }],
  histogram: Array(256).fill(0),
  printableBytes: 60,
  nulBytes: 0,
  highBitBytes: 0,
  printableRatio: 0.9375,
  isLikelyText: true,
  asciiPreview: "%PDF-1.7",
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
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "viewer_analyze_path") return mockSourceAnalysis;
        if (cmd === "viewer_detect_type") {
          return {
            mime_type: "application/pdf",
            description: "PDF Document",
            extension: "pdf",
            is_text: true,
            is_forensic_format: false,
            magic_hex: "25 50 44 46 2D",
          };
        }
        if (cmd === "viewer_parse_header") return {
          format: "PDF Document",
          version: "1.7",
          regions: [],
          fields: [],
        };
        return undefined;
      });

      const onMetadata = vi.fn();

      renderComponent(() => (
        <HexViewer file={mockDiskFile} onMetadataLoaded={onMetadata} />
      ));
      await tick(200);

      expect(mockReadBytesFromSource).toHaveBeenCalled();
      expect(mockInvoke).toHaveBeenCalledWith("viewer_analyze_path", {
        path: mockDiskFile.path,
        options: { offset: 0, length: 65536, entropyWindowBytes: 4096 },
      });
      expect(onMetadata).toHaveBeenCalled();
      const lastMetadataCall = onMetadata.mock.calls[onMetadata.mock.calls.length - 1];
      expect(lastMetadataCall?.[0].fields).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ key: "Entropy", value: "4.250 bits/byte" }),
          expect.objectContaining({ key: "Magic Bytes", value: "25 50 44 46 2D" }),
          expect.objectContaining({
            key: "Embedded Signatures",
            value: "PNG Image @ 0x80",
          }),
        ]),
      );
      expect(lastMetadataCall?.[0].regions).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            start: 128,
            end: 136,
            name: "Embedded Signature 1",
            description: "PNG Image (image/png) at 0x80",
          }),
        ]),
      );
    });

    it("analyzes container entries through source-aware viewer command", async () => {
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: createMockBytes(64),
        totalSize: 262144,
      });
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "viewer_analyze_source") return {
          ...mockSourceAnalysis,
          sourceRef: {
            kind: "containerEntry",
            containerPath: mockContainerEntry.containerPath,
            entryPath: mockContainerEntry.entryPath,
            containerType: "ad1",
          },
          sourceId: "ad1:/evidence/image.ad1:System32/config/SAM",
          totalSize: mockContainerEntry.size,
        };
        return undefined;
      });

      const onMetadata = vi.fn();
      renderComponent(() => (
        <HexViewer entry={mockContainerEntry} onMetadataLoaded={onMetadata} />
      ));
      await tick(200);

      expect(mockInvoke).toHaveBeenCalledWith("viewer_analyze_source", {
        source: {
          containerPath: mockContainerEntry.containerPath,
          entryPath: mockContainerEntry.entryPath,
          containerType: "ad1",
          size: mockContainerEntry.size,
        },
        options: { offset: 0, length: 65536, entropyWindowBytes: 4096 },
      });
      expect(onMetadata).toHaveBeenCalledWith(
        expect.objectContaining({
          format: "PDF Document",
        }),
      );
    });

    it("persists disk-file source analysis when project DB is open", async () => {
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: createMockBytes(64),
        totalSize: 4096,
      });
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "project_db_is_open") return true;
        if (cmd === "project_db_analyze_source_and_insert") {
          return {
            analysis: mockSourceAnalysis,
            record: {
              id: "analysis-1",
              sourceId: mockSourceAnalysis.sourceId,
              sourceRefJson: JSON.stringify(mockSourceAnalysis.sourceRef),
            },
          };
        }
        return undefined;
      });

      renderComponent(() => <HexViewer file={mockDiskFile} />);
      await tick(200);

      expect(mockInvoke).toHaveBeenCalledWith("project_db_analyze_source_and_insert", {
        request: {
          source: {
            path: mockDiskFile.path,
            containerType: "disk",
            size: mockDiskFile.size,
          },
          options: { offset: 0, length: 65536, entropyWindowBytes: 4096 },
          evidenceFile: {
            id: mockDiskFile.path,
            path: mockDiskFile.path,
            filename: mockDiskFile.filename,
            containerType: mockDiskFile.container_type,
            totalSize: mockDiskFile.size,
            segmentCount: 1,
            discoveredAt: expect.any(String),
            created: null,
            modified: null,
          },
          analyzer: "hex-viewer",
        },
      });
      expect(mockInvoke).not.toHaveBeenCalledWith("viewer_analyze_path", expect.anything());
    });

    it("falls back to transient source analysis when persistence is unavailable", async () => {
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: createMockBytes(64),
        totalSize: mockContainerEntry.size,
      });
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "project_db_is_open") return true;
        if (cmd === "project_db_analyze_source_and_insert") {
          throw new Error("No project DB open");
        }
        if (cmd === "viewer_analyze_source") return mockSourceAnalysis;
        return undefined;
      });

      renderComponent(() => <HexViewer entry={mockContainerEntry} />);
      await tick(200);

      expect(mockInvoke).toHaveBeenCalledWith("project_db_analyze_source_and_insert", {
        request: expect.objectContaining({
          source: {
            containerPath: mockContainerEntry.containerPath,
            entryPath: mockContainerEntry.entryPath,
            containerType: "ad1",
            size: mockContainerEntry.size,
          },
          analyzer: "hex-viewer",
        }),
      });
      expect(mockInvoke).toHaveBeenCalledWith("viewer_analyze_source", {
        source: {
          containerPath: mockContainerEntry.containerPath,
          entryPath: mockContainerEntry.entryPath,
          containerType: "ad1",
          size: mockContainerEntry.size,
        },
        options: { offset: 0, length: 65536, entropyWindowBytes: 4096 },
      });
    });

    it("analyzes nested archive paths through source-aware viewer command", async () => {
      const nestedEntry = {
        ...mockContainerEntry,
        isArchiveEntry: true,
        entryPath: "inner.zip::nested.txt",
        containerType: "zip",
      };
      mockReadBytesFromSource.mockResolvedValueOnce({
        bytes: createMockBytes(64),
        totalSize: nestedEntry.size,
      });
      mockInvoke.mockImplementation(async (cmd: string) => {
        if (cmd === "viewer_analyze_source") return {
          ...mockSourceAnalysis,
          sourceRef: {
            kind: "nestedContainerEntry",
            containerPath: nestedEntry.containerPath,
            nestedContainerPath: "inner.zip",
            entryPath: "nested.txt",
            containerType: "zip",
          },
        };
        return undefined;
      });

      renderComponent(() => <HexViewer entry={nestedEntry} />);
      await tick(200);

      expect(mockInvoke).toHaveBeenCalledWith("viewer_analyze_source", {
        source: {
          containerPath: nestedEntry.containerPath,
          nestedArchivePath: "inner.zip",
          entryPath: "nested.txt",
          containerType: "zip",
          size: nestedEntry.size,
        },
        options: { offset: 0, length: 65536, entropyWindowBytes: 4096 },
      });
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
