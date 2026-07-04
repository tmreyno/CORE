// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

import {
  readBytesFromSource,
  readTextFromSource,
  getSourceKey,
  getSourceFilename,
} from "../useEntrySource";
import type { DiscoveredFile } from "../../types";
import type { SelectedEntry } from "../../components/EvidenceTree/types";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const makeFile = (path: string, size = 1024): DiscoveredFile => ({
  path,
  filename: path.split("/").pop() || path,
  size,
  container_type: "ad1",
});

const makeEntry = (overrides: Partial<SelectedEntry> = {}): SelectedEntry => ({
  containerPath: "/evidence/container.ad1",
  entryPath: "/files/test.bin",
  name: "test.bin",
  size: 2048,
  isDir: false,
  isArchiveEntry: false,
  isVfsEntry: false,
  isDiskFile: false,
  ...overrides,
});

const makeChunk = (bytes: number[], totalSize = bytes.length, offset = 0) => ({
  path: "source",
  offset,
  bytesRead: bytes.length,
  totalSize,
  eof: offset + bytes.length >= totalSize,
  data: globalThis.btoa(String.fromCharCode(...bytes)),
});

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

beforeEach(() => {
  vi.clearAllMocks();
});

// ---------------------------------------------------------------------------
// getSourceKey
// ---------------------------------------------------------------------------

describe("getSourceKey", () => {
  it("returns entry key for SelectedEntry", () => {
    const entry = makeEntry({
      containerPath: "/evidence/disk.ad1",
      entryPath: "/files/doc.pdf",
    });
    expect(getSourceKey(null, entry)).toBe("entry:/evidence/disk.ad1:/files/doc.pdf");
  });

  it("returns file key for DiscoveredFile", () => {
    const file = makeFile("/evidence/disk.e01");
    expect(getSourceKey(file, undefined)).toBe("file:/evidence/disk.e01");
  });

  it("prefers entry over file when both provided", () => {
    const file = makeFile("/evidence/disk.e01");
    const entry = makeEntry();
    expect(getSourceKey(file, entry)).toBe(
      "entry:/evidence/container.ad1:/files/test.bin"
    );
  });

  it("returns null when neither provided", () => {
    expect(getSourceKey(null, undefined)).toBeNull();
  });

  it("returns null for undefined file", () => {
    expect(getSourceKey(undefined, undefined)).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// getSourceFilename
// ---------------------------------------------------------------------------

describe("getSourceFilename", () => {
  it("returns entry name when entry is provided", () => {
    const entry = makeEntry({ name: "evidence.doc" });
    expect(getSourceFilename(null, entry)).toBe("evidence.doc");
  });

  it("returns file filename when file is provided", () => {
    const file = makeFile("/path/to/report.pdf");
    expect(getSourceFilename(file, undefined)).toBe("report.pdf");
  });

  it("prefers entry name over file name", () => {
    const file = makeFile("/path/to/report.pdf");
    const entry = makeEntry({ name: "entry-name.bin" });
    expect(getSourceFilename(file, entry)).toBe("entry-name.bin");
  });

  it("returns empty string when neither provided", () => {
    expect(getSourceFilename(null, undefined)).toBe("");
  });

  it("returns empty string for undefined file", () => {
    expect(getSourceFilename(undefined, undefined)).toBe("");
  });
});

// ---------------------------------------------------------------------------
// readBytesFromSource
// ---------------------------------------------------------------------------

describe("readBytesFromSource", () => {
  it("reads from VFS entry using the source byte command", async () => {
    const entry = makeEntry({
      containerPath: "/evidence/disk.e01",
      isVfsEntry: true,
      size: 4096,
    });
    mockInvoke.mockResolvedValueOnce(makeChunk([0x48, 0x65, 0x6c, 0x6c, 0x6f], 4096));

    const result = await readBytesFromSource(null, entry, 0, 256);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_source_base64_chunk", {
      source: expect.objectContaining({
        containerPath: "/evidence/disk.e01",
        entryPath: entry.entryPath,
        containerType: "e01",
        size: 4096,
      }),
      offset: 0,
      size: 256,
    });
    expect(result.bytes).toEqual([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    expect(result.totalSize).toBe(4096);
  });

  it("reads from archive entry using the source byte command", async () => {
    const entry = makeEntry({
      containerPath: "/evidence/archive.zip",
      isArchiveEntry: true,
      size: 512,
    });
    mockInvoke.mockResolvedValueOnce(makeChunk([0xff, 0xfe], 512, 100));

    const result = await readBytesFromSource(null, entry, 100, 128);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_source_base64_chunk", {
      source: expect.objectContaining({
        containerPath: "/evidence/archive.zip",
        entryPath: entry.entryPath,
        containerType: "zip",
        size: 512,
      }),
      offset: 100,
      size: 128,
    });
    expect(result.bytes).toEqual([0xff, 0xfe]);
    expect(result.totalSize).toBe(512);
  });

  it("reads from nested archive entry using the source byte command", async () => {
    const entry = makeEntry({
      containerPath: "/evidence/archive.zip",
      isArchiveEntry: true,
      entryPath: "inner.zip::file.txt",
      size: 1024,
    });
    mockInvoke.mockResolvedValueOnce(makeChunk([0x01, 0x02], 1024));

    const result = await readBytesFromSource(null, entry, 0, 64);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_source_base64_chunk", {
      source: expect.objectContaining({
        containerPath: "/evidence/archive.zip",
        nestedArchivePath: "inner.zip",
        entryPath: "file.txt",
        containerType: "zip",
        size: 1024,
      }),
      offset: 0,
      size: 64,
    });
    expect(result.bytes).toEqual([0x01, 0x02]);
    expect(result.totalSize).toBe(1024);
  });

  it("reads from disk file entry using the source byte command", async () => {
    const entry = makeEntry({ isDiskFile: true, size: 8192 });
    mockInvoke.mockResolvedValueOnce(makeChunk([0xAA, 0xBB], 8192));

    const result = await readBytesFromSource(null, entry, 0, 256);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_source_base64_chunk", {
      source: expect.objectContaining({
        path: entry.entryPath,
        entryPath: entry.entryPath,
        containerType: "disk",
        size: 8192,
      }),
      offset: 0,
      size: 256,
    });
    expect(result.bytes).toEqual([0xAA, 0xBB]);
    expect(result.totalSize).toBe(8192);
  });

  it("reads from AD1 container entry using the source byte command", async () => {
    const entry = makeEntry(); // no special flags = AD1
    mockInvoke.mockResolvedValueOnce(makeChunk([0xDE, 0xAD], 2048, 512));

    const result = await readBytesFromSource(null, entry, 512, 128);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_source_base64_chunk", {
      source: expect.objectContaining({
        containerPath: entry.containerPath,
        entryPath: entry.entryPath,
        containerType: "ad1",
        size: 2048,
      }),
      offset: 512,
      size: 128,
    });
    expect(result.bytes).toEqual([0xDE, 0xAD]);
    expect(result.totalSize).toBe(2048);
  });

  it("reads from disk file (DiscoveredFile) using the source byte command", async () => {
    const file = makeFile("/evidence/disk.e01", 10000);
    mockInvoke.mockResolvedValueOnce(makeChunk([0x50, 0x4B], 10000));

    const result = await readBytesFromSource(file, undefined, 0, 512);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_source_base64_chunk", {
      source: expect.objectContaining({
        path: file.path,
        containerType: "disk",
        size: 10000,
      }),
      offset: 0,
      size: 512,
    });
    expect(result.bytes).toEqual([0x50, 0x4B]);
    expect(result.totalSize).toBe(10000);
  });

  it("throws when neither file nor entry provided", async () => {
    await expect(readBytesFromSource(null, undefined, 0, 256)).rejects.toThrow(
      "No file or entry provided"
    );
  });

  it("prioritizes entry over file when both provided", async () => {
    const file = makeFile("/evidence/disk.e01");
    const entry = makeEntry({ isVfsEntry: true });
    mockInvoke.mockResolvedValueOnce(makeChunk([0x01], 2048));

    await readBytesFromSource(file, entry, 0, 16);

    expect(mockInvoke).toHaveBeenCalledWith(
      "viewer_read_binary_source_base64_chunk",
      expect.objectContaining({
        source: expect.objectContaining({
          containerPath: entry.containerPath,
          entryPath: entry.entryPath,
        }),
      })
    );
  });

  it("uses explicit concrete container type over generic UI type", async () => {
    const entry = makeEntry({
      containerPath: "/evidence/disk.raw",
      isVfsEntry: true,
      containerType: "vfs",
    });
    mockInvoke.mockResolvedValueOnce(makeChunk([0x01], 2048));

    await readBytesFromSource(null, entry, 0, 16);

    expect(mockInvoke).toHaveBeenCalledWith(
      "viewer_read_binary_source_base64_chunk",
      expect.objectContaining({
        source: expect.objectContaining({
          containerType: "raw",
        }),
      })
    );
  });

  it("passes UFED lazy-tree entries through the source byte command", async () => {
    const entry = makeEntry({
      containerPath: "/evidence/mobile.zip",
      entryPath: "files/messages.db",
      containerType: "ufed",
      size: 4096,
    });
    mockInvoke.mockResolvedValueOnce(makeChunk([0x53, 0x51, 0x4c], 4096));

    const result = await readBytesFromSource(null, entry, 0, 3);

    expect(result.bytes).toEqual([0x53, 0x51, 0x4c]);
    expect(mockInvoke).toHaveBeenCalledWith(
      "viewer_read_binary_source_base64_chunk",
      expect.objectContaining({
        source: expect.objectContaining({
          containerPath: "/evidence/mobile.zip",
          entryPath: "files/messages.db",
          containerType: "ufed",
          size: 4096,
        }),
        offset: 0,
        size: 3,
      })
    );
  });
});

// ---------------------------------------------------------------------------
// readTextFromSource
// ---------------------------------------------------------------------------

describe("readTextFromSource", () => {
  it("reads text from entry by decoding bytes", async () => {
    const entry = makeEntry({ isVfsEntry: true, size: 100 });
    // "Hello" in UTF-8
    mockInvoke.mockResolvedValueOnce(makeChunk([0x48, 0x65, 0x6c, 0x6c, 0x6f], 100));

    const result = await readTextFromSource(null, entry, 0, 256);

    expect(result.text).toBe("Hello");
    expect(result.totalSize).toBe(100);
  });

  it("reads text from disk file using the source byte command", async () => {
    const file = makeFile("/evidence/notes.txt", 500);
    const bytes = Array.from(new TextEncoder().encode("File contents here"));
    mockInvoke.mockResolvedValueOnce(makeChunk(bytes, 500));

    const result = await readTextFromSource(file, undefined, 0, 1024);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_binary_source_base64_chunk", {
      source: expect.objectContaining({
        path: file.path,
        containerType: "disk",
        size: 500,
      }),
      offset: 0,
      size: 4096,
    });
    expect(result.text).toBe("File contents here");
    expect(result.totalSize).toBe(500);
  });

  it("throws when neither file nor entry provided", async () => {
    await expect(readTextFromSource(null, undefined, 0, 256)).rejects.toThrow(
      "No file or entry provided"
    );
  });

  it("handles non-UTF8 bytes gracefully", async () => {
    const entry = makeEntry({ size: 4 });
    mockInvoke.mockResolvedValueOnce(makeChunk([0xFF, 0xFE, 0x00, 0x01], 4));

    const result = await readTextFromSource(null, entry, 0, 4);

    // TextDecoder with fatal:false replaces invalid sequences
    expect(typeof result.text).toBe("string");
    expect(result.totalSize).toBe(4);
  });

  it("truncates decoded text to maxChars", async () => {
    const entry = makeEntry({ size: 10 });
    const bytes = Array.from(new TextEncoder().encode("abcdef"));
    mockInvoke.mockResolvedValueOnce(makeChunk(bytes, 10));

    const result = await readTextFromSource(null, entry, 0, 3);

    expect(result.text).toBe("abc");
    expect(mockInvoke).toHaveBeenCalledWith(
      "viewer_read_binary_source_base64_chunk",
      expect.objectContaining({ size: 12 })
    );
  });
});
