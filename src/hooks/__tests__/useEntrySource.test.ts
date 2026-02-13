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
  it("reads from VFS entry using vfs_read_file", async () => {
    const entry = makeEntry({ isVfsEntry: true, size: 4096 });
    mockInvoke.mockResolvedValueOnce([0x48, 0x65, 0x6c, 0x6c, 0x6f]);

    const result = await readBytesFromSource(null, entry, 0, 256);

    expect(mockInvoke).toHaveBeenCalledWith("vfs_read_file", {
      containerPath: entry.containerPath,
      filePath: entry.entryPath,
      offset: 0,
      length: 256,
    });
    expect(result.bytes).toEqual([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    expect(result.totalSize).toBe(4096);
  });

  it("reads from archive entry using archive_read_entry_chunk", async () => {
    const entry = makeEntry({ isArchiveEntry: true, size: 512 });
    mockInvoke.mockResolvedValueOnce([0xff, 0xfe]);

    const result = await readBytesFromSource(null, entry, 100, 128);

    expect(mockInvoke).toHaveBeenCalledWith("archive_read_entry_chunk", {
      containerPath: entry.containerPath,
      entryPath: entry.entryPath,
      offset: 100,
      size: 128,
    });
    expect(result.bytes).toEqual([0xff, 0xfe]);
    expect(result.totalSize).toBe(512);
  });

  it("reads from nested archive entry using nested_archive_read_entry_chunk", async () => {
    const entry = makeEntry({
      isArchiveEntry: true,
      entryPath: "inner.zip::file.txt",
      size: 1024,
    });
    mockInvoke.mockResolvedValueOnce([0x01, 0x02]);

    const result = await readBytesFromSource(null, entry, 0, 64);

    expect(mockInvoke).toHaveBeenCalledWith("nested_archive_read_entry_chunk", {
      containerPath: entry.containerPath,
      nestedArchivePath: "inner.zip",
      entryPath: "file.txt",
      offset: 0,
      size: 64,
    });
    expect(result.bytes).toEqual([0x01, 0x02]);
    expect(result.totalSize).toBe(1024);
  });

  it("reads from disk file entry using read_file_bytes", async () => {
    const entry = makeEntry({ isDiskFile: true, size: 8192 });
    mockInvoke.mockResolvedValueOnce([0xAA, 0xBB]);

    const result = await readBytesFromSource(null, entry, 0, 256);

    expect(mockInvoke).toHaveBeenCalledWith("read_file_bytes", {
      path: entry.entryPath,
      offset: 0,
      length: 256,
    });
    expect(result.bytes).toEqual([0xAA, 0xBB]);
    expect(result.totalSize).toBe(8192);
  });

  it("reads from AD1 container entry using container_read_entry_chunk", async () => {
    const entry = makeEntry(); // no special flags = AD1
    mockInvoke.mockResolvedValueOnce([0xDE, 0xAD]);

    const result = await readBytesFromSource(null, entry, 512, 128);

    expect(mockInvoke).toHaveBeenCalledWith("container_read_entry_chunk", {
      containerPath: entry.containerPath,
      entryPath: entry.entryPath,
      offset: 512,
      size: 128,
    });
    expect(result.bytes).toEqual([0xDE, 0xAD]);
    expect(result.totalSize).toBe(2048);
  });

  it("reads from disk file (DiscoveredFile) using viewer_read_chunk", async () => {
    const file = makeFile("/evidence/disk.e01", 10000);
    mockInvoke.mockResolvedValueOnce({ bytes: [0x50, 0x4B], total_size: 10000 });

    const result = await readBytesFromSource(file, undefined, 0, 512);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_chunk", {
      path: file.path,
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
    mockInvoke.mockResolvedValueOnce([0x01]);

    await readBytesFromSource(file, entry, 0, 16);

    expect(mockInvoke).toHaveBeenCalledWith("vfs_read_file", expect.any(Object));
  });

  it("handles VFS entry priority over archive flag", async () => {
    // VFS takes precedence over archive in the if-chain
    const entry = makeEntry({ isVfsEntry: true, isArchiveEntry: true });
    mockInvoke.mockResolvedValueOnce([0x01]);

    await readBytesFromSource(null, entry, 0, 16);

    expect(mockInvoke).toHaveBeenCalledWith("vfs_read_file", expect.any(Object));
  });
});

// ---------------------------------------------------------------------------
// readTextFromSource
// ---------------------------------------------------------------------------

describe("readTextFromSource", () => {
  it("reads text from entry by decoding bytes", async () => {
    const entry = makeEntry({ isVfsEntry: true, size: 100 });
    // "Hello" in UTF-8
    mockInvoke.mockResolvedValueOnce([0x48, 0x65, 0x6c, 0x6c, 0x6f]);

    const result = await readTextFromSource(null, entry, 0, 256);

    expect(result.text).toBe("Hello");
    expect(result.totalSize).toBe(100);
  });

  it("reads text from disk file using viewer_read_text", async () => {
    const file = makeFile("/evidence/notes.txt", 500);
    mockInvoke.mockResolvedValueOnce("File contents here");

    const result = await readTextFromSource(file, undefined, 0, 1024);

    expect(mockInvoke).toHaveBeenCalledWith("viewer_read_text", {
      path: file.path,
      offset: 0,
      maxChars: 1024,
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
    mockInvoke.mockResolvedValueOnce([0xFF, 0xFE, 0x00, 0x01]);

    const result = await readTextFromSource(null, entry, 0, 4);

    // TextDecoder with fatal:false replaces invalid sequences
    expect(typeof result.text).toBe("string");
    expect(result.totalSize).toBe(4);
  });
});
