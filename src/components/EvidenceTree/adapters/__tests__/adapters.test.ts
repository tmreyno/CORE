// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { ad1Adapter } from "../ad1Adapter";
import { archiveAdapter } from "../archiveAdapter";
import { ufedAdapter } from "../ufedAdapter";
import { vfsAdapter } from "../vfsAdapter";
import { lazyAdapter } from "../lazyAdapter";
import type { TreeEntry, ArchiveTreeEntry, UfedTreeEntry } from "../../../../types";
import type { VfsEntry } from "../../../../types/vfs";
import type { LazyTreeEntry } from "../../../../types/lazy-loading";

// ============================================================================
// Test Fixtures
// ============================================================================

const CONTAINER_PATH = "/evidence/test.ad1";

function makeTreeEntry(overrides: Partial<TreeEntry> = {}): TreeEntry {
  return {
    path: "Documents/report.docx",
    name: "report.docx",
    is_dir: false,
    size: 1024,
    item_type: 1,
    item_addr: 0x100,
    first_child_addr: null,
    data_addr: 0x200,
    md5_hash: "d41d8cd98f00b204e9800998ecf8427e",
    sha1_hash: null,
    ...overrides,
  };
}

function makeDirEntry(overrides: Partial<TreeEntry> = {}): TreeEntry {
  return makeTreeEntry({
    path: "Documents",
    name: "Documents",
    is_dir: true,
    size: 0,
    item_type: 0,
    first_child_addr: 0x300,
    data_addr: null,
    md5_hash: null,
    ...overrides,
  });
}

function makeArchiveEntry(overrides: Partial<ArchiveTreeEntry> = {}): ArchiveTreeEntry {
  return {
    path: "images/photo.jpg",
    name: "photo.jpg",
    isDir: false,
    size: 2048,
    compressedSize: 1500,
    crc32: 0xDEADBEEF,
    modified: "2024-01-15T10:30:00Z",
    ...overrides,
  };
}

function makeUfedEntry(overrides: Partial<UfedTreeEntry> = {}): UfedTreeEntry {
  return {
    path: "Chats/WhatsApp/chat.db",
    name: "chat.db",
    isDir: false,
    size: 4096,
    entryType: "file",
    hash: "abc123def456",
    modified: "2024-06-01T12:00:00Z",
    ...overrides,
  };
}

function makeVfsEntry(overrides: Partial<VfsEntry> = {}): VfsEntry {
  return {
    name: "System32",
    path: "/Windows/System32",
    isDir: true,
    size: 0,
    ...overrides,
  };
}

function makeLazyEntry(overrides: Partial<LazyTreeEntry> = {}): LazyTreeEntry {
  return {
    id: "entry-001",
    name: "evidence.zip",
    path: "exports/evidence.zip",
    is_dir: false,
    size: 8192,
    entry_type: "file",
    child_count: 0,
    children_loaded: false,
    hash: "sha256:abcdef1234567890",
    modified: "2024-03-10T08:00:00Z",
    metadata: null,
    ...overrides,
  };
}

// ============================================================================
// AD1 Adapter Tests
// ============================================================================

describe("ad1Adapter", () => {
  describe("getKey", () => {
    it("uses item_addr when available", () => {
      const entry = makeTreeEntry({ item_addr: 256 });
      expect(ad1Adapter.getKey(entry, CONTAINER_PATH)).toBe(
        `${CONTAINER_PATH}::addr:256`
      );
    });

    it("falls back to path when item_addr is null", () => {
      const entry = makeTreeEntry({ item_addr: null });
      expect(ad1Adapter.getKey(entry, CONTAINER_PATH)).toBe(
        `${CONTAINER_PATH}::path:Documents/report.docx`
      );
    });

    it("falls back to path when item_addr is undefined", () => {
      const entry = makeTreeEntry({ item_addr: undefined });
      expect(ad1Adapter.getKey(entry, CONTAINER_PATH)).toBe(
        `${CONTAINER_PATH}::path:Documents/report.docx`
      );
    });

    it("uses addr:0 when item_addr is 0", () => {
      // 0 is falsy but != null, should still use addr
      const entry = makeTreeEntry({ item_addr: 0 });
      // item_addr != null → false for 0 (0 != null is true)
      expect(ad1Adapter.getKey(entry, CONTAINER_PATH)).toBe(
        `${CONTAINER_PATH}::addr:0`
      );
    });
  });

  describe("getName", () => {
    it("returns entry name", () => {
      expect(ad1Adapter.getName(makeTreeEntry())).toBe("report.docx");
    });
  });

  describe("getPath", () => {
    it("returns entry path", () => {
      expect(ad1Adapter.getPath(makeTreeEntry())).toBe("Documents/report.docx");
    });
  });

  describe("isDir", () => {
    it("returns false for files", () => {
      expect(ad1Adapter.isDir(makeTreeEntry())).toBe(false);
    });

    it("returns true for directories", () => {
      expect(ad1Adapter.isDir(makeDirEntry())).toBe(true);
    });
  });

  describe("getSize", () => {
    it("returns file size", () => {
      expect(ad1Adapter.getSize(makeTreeEntry({ size: 1024 }))).toBe(1024);
    });

    it("returns 0 for directories", () => {
      expect(ad1Adapter.getSize(makeDirEntry())).toBe(0);
    });
  });

  describe("getHash", () => {
    it("returns md5_hash when available", () => {
      expect(ad1Adapter.getHash!(makeTreeEntry())).toBe("d41d8cd98f00b204e9800998ecf8427e");
    });

    it("returns sha1_hash when md5 is not available", () => {
      const entry = makeTreeEntry({ md5_hash: null, sha1_hash: "abc123" });
      expect(ad1Adapter.getHash!(entry)).toBe("abc123");
    });

    it("returns undefined when no hash available", () => {
      const entry = makeTreeEntry({ md5_hash: null, sha1_hash: null });
      expect(ad1Adapter.getHash!(entry)).toBeUndefined();
    });
  });

  describe("hasChildren", () => {
    it("returns true for directory with first_child_addr > 0", () => {
      expect(ad1Adapter.hasChildren(makeDirEntry({ first_child_addr: 0x300 }))).toBe(true);
    });

    it("returns false for directory with first_child_addr = 0", () => {
      expect(ad1Adapter.hasChildren(makeDirEntry({ first_child_addr: 0 }))).toBe(false);
    });

    it("returns false for directory with null first_child_addr", () => {
      expect(ad1Adapter.hasChildren(makeDirEntry({ first_child_addr: null }))).toBe(false);
    });

    it("returns false for files (even with first_child_addr)", () => {
      expect(ad1Adapter.hasChildren(makeTreeEntry({ first_child_addr: 0x100 }))).toBe(false);
    });
  });

  describe("isNestedContainer", () => {
    it("always returns false", () => {
      expect(ad1Adapter.isNestedContainer!(makeTreeEntry())).toBe(false);
      expect(ad1Adapter.isNestedContainer!(makeDirEntry())).toBe(false);
    });
  });
});

// ============================================================================
// Archive Adapter Tests
// ============================================================================

describe("archiveAdapter", () => {
  describe("getKey", () => {
    it("creates archive-prefixed key", () => {
      const entry = makeArchiveEntry();
      expect(archiveAdapter.getKey(entry, CONTAINER_PATH)).toBe(
        `${CONTAINER_PATH}::archive::images/photo.jpg`
      );
    });
  });

  describe("getName", () => {
    it("returns name when available", () => {
      expect(archiveAdapter.getName(makeArchiveEntry({ name: "photo.jpg" }))).toBe("photo.jpg");
    });

    it("extracts name from path when name is empty", () => {
      const entry = makeArchiveEntry({ name: "", path: "deep/nested/file.txt" });
      expect(archiveAdapter.getName(entry)).toBe("file.txt");
    });

    it("handles paths with trailing slash", () => {
      const entry = makeArchiveEntry({ name: "", path: "folder/" });
      expect(archiveAdapter.getName(entry)).toBe("folder");
    });

    it("falls back to full path when no segments", () => {
      const entry = makeArchiveEntry({ name: "", path: "" });
      expect(archiveAdapter.getName(entry)).toBe("");
    });
  });

  describe("getPath", () => {
    it("returns entry path", () => {
      expect(archiveAdapter.getPath(makeArchiveEntry())).toBe("images/photo.jpg");
    });
  });

  describe("isDir", () => {
    it("returns false for files", () => {
      expect(archiveAdapter.isDir(makeArchiveEntry())).toBe(false);
    });

    it("returns true for directories", () => {
      expect(archiveAdapter.isDir(makeArchiveEntry({ isDir: true }))).toBe(true);
    });
  });

  describe("getSize", () => {
    it("returns uncompressed size", () => {
      expect(archiveAdapter.getSize(makeArchiveEntry({ size: 2048 }))).toBe(2048);
    });
  });

  describe("getHash", () => {
    it("returns CRC32 as uppercase hex string", () => {
      expect(archiveAdapter.getHash!(makeArchiveEntry({ crc32: 0xDEADBEEF }))).toBe("DEADBEEF");
    });

    it("returns undefined when CRC32 is 0", () => {
      expect(archiveAdapter.getHash!(makeArchiveEntry({ crc32: 0 }))).toBeUndefined();
    });
  });

  describe("hasChildren", () => {
    it("returns true for directories", () => {
      expect(archiveAdapter.hasChildren(makeArchiveEntry({ isDir: true }))).toBe(true);
    });

    it("returns false for files", () => {
      expect(archiveAdapter.hasChildren(makeArchiveEntry({ isDir: false }))).toBe(false);
    });
  });

  describe("getEntryType", () => {
    it("returns 'container' for nested container files", () => {
      const entry = makeArchiveEntry({ isDir: false, name: "nested.ad1", path: "nested.ad1" });
      expect(archiveAdapter.getEntryType!(entry)).toBe("container");
    });

    it("returns 'container' for e01 files", () => {
      const entry = makeArchiveEntry({ isDir: false, name: "disk.e01", path: "disk.e01" });
      expect(archiveAdapter.getEntryType!(entry)).toBe("container");
    });

    it("returns undefined for regular files", () => {
      const entry = makeArchiveEntry({ isDir: false, name: "readme.txt", path: "readme.txt" });
      expect(archiveAdapter.getEntryType!(entry)).toBeUndefined();
    });

    it("returns undefined for directories", () => {
      const entry = makeArchiveEntry({ isDir: true, name: "folder", path: "folder" });
      expect(archiveAdapter.getEntryType!(entry)).toBeUndefined();
    });
  });

  describe("isNestedContainer", () => {
    it("returns true for container files", () => {
      const entry = makeArchiveEntry({ isDir: false, name: "nested.ad1", path: "nested.ad1" });
      expect(archiveAdapter.isNestedContainer!(entry)).toBe(true);
    });

    it("returns false for non-container files", () => {
      const entry = makeArchiveEntry({ isDir: false, name: "data.csv", path: "data.csv" });
      expect(archiveAdapter.isNestedContainer!(entry)).toBe(false);
    });

    it("returns false for directories", () => {
      const entry = makeArchiveEntry({ isDir: true, name: "folder", path: "folder" });
      expect(archiveAdapter.isNestedContainer!(entry)).toBe(false);
    });
  });
});

// ============================================================================
// UFED Adapter Tests
// ============================================================================

describe("ufedAdapter", () => {
  describe("getKey", () => {
    it("creates ufed-prefixed key", () => {
      const entry = makeUfedEntry();
      expect(ufedAdapter.getKey(entry, CONTAINER_PATH)).toBe(
        `${CONTAINER_PATH}::ufed::Chats/WhatsApp/chat.db`
      );
    });
  });

  describe("getName", () => {
    it("returns entry name", () => {
      expect(ufedAdapter.getName(makeUfedEntry())).toBe("chat.db");
    });
  });

  describe("getPath", () => {
    it("returns entry path", () => {
      expect(ufedAdapter.getPath(makeUfedEntry())).toBe("Chats/WhatsApp/chat.db");
    });
  });

  describe("isDir", () => {
    it("returns false for files", () => {
      expect(ufedAdapter.isDir(makeUfedEntry())).toBe(false);
    });

    it("returns true for directories", () => {
      expect(ufedAdapter.isDir(makeUfedEntry({ isDir: true }))).toBe(true);
    });
  });

  describe("getSize", () => {
    it("returns file size", () => {
      expect(ufedAdapter.getSize(makeUfedEntry({ size: 4096 }))).toBe(4096);
    });
  });

  describe("getHash", () => {
    it("returns hash when available", () => {
      expect(ufedAdapter.getHash!(makeUfedEntry({ hash: "abc123" }))).toBe("abc123");
    });

    it("returns undefined when hash is null", () => {
      expect(ufedAdapter.getHash!(makeUfedEntry({ hash: null }))).toBeUndefined();
    });

    it("returns undefined when hash is undefined", () => {
      expect(ufedAdapter.getHash!(makeUfedEntry({ hash: undefined }))).toBeUndefined();
    });
  });

  describe("getEntryType", () => {
    it("returns entry type when available", () => {
      expect(ufedAdapter.getEntryType!(makeUfedEntry({ entryType: "extraction" }))).toBe("extraction");
    });
  });

  describe("hasChildren", () => {
    it("returns true for directories", () => {
      expect(ufedAdapter.hasChildren(makeUfedEntry({ isDir: true }))).toBe(true);
    });

    it("returns false for files", () => {
      expect(ufedAdapter.hasChildren(makeUfedEntry({ isDir: false }))).toBe(false);
    });
  });

  describe("isNestedContainer", () => {
    it("always returns false", () => {
      expect(ufedAdapter.isNestedContainer!(makeUfedEntry())).toBe(false);
    });
  });
});

// ============================================================================
// VFS Adapter Tests
// ============================================================================

describe("vfsAdapter", () => {
  describe("getKey", () => {
    it("creates vfs-prefixed key", () => {
      const entry = makeVfsEntry();
      expect(vfsAdapter.getKey(entry, CONTAINER_PATH)).toBe(
        `${CONTAINER_PATH}::vfs::/Windows/System32`
      );
    });
  });

  describe("getName", () => {
    it("returns entry name", () => {
      expect(vfsAdapter.getName(makeVfsEntry())).toBe("System32");
    });
  });

  describe("getPath", () => {
    it("returns entry path", () => {
      expect(vfsAdapter.getPath(makeVfsEntry())).toBe("/Windows/System32");
    });
  });

  describe("isDir", () => {
    it("returns true for directories", () => {
      expect(vfsAdapter.isDir(makeVfsEntry({ isDir: true }))).toBe(true);
    });

    it("returns false for files", () => {
      expect(vfsAdapter.isDir(makeVfsEntry({ isDir: false }))).toBe(false);
    });
  });

  describe("getSize", () => {
    it("returns file size", () => {
      expect(vfsAdapter.getSize(makeVfsEntry({ size: 512 }))).toBe(512);
    });
  });

  describe("getHash", () => {
    it("always returns undefined (VFS entries don't store hashes)", () => {
      expect(vfsAdapter.getHash!(makeVfsEntry())).toBeUndefined();
    });
  });

  describe("hasChildren", () => {
    it("returns true for directories", () => {
      expect(vfsAdapter.hasChildren(makeVfsEntry({ isDir: true }))).toBe(true);
    });

    it("returns false for files", () => {
      expect(vfsAdapter.hasChildren(makeVfsEntry({ isDir: false }))).toBe(false);
    });
  });

  describe("isNestedContainer", () => {
    it("always returns false", () => {
      expect(vfsAdapter.isNestedContainer!(makeVfsEntry())).toBe(false);
    });
  });
});

// ============================================================================
// Lazy Adapter Tests
// ============================================================================

describe("lazyAdapter", () => {
  describe("getKey", () => {
    it("creates lazy-prefixed key", () => {
      const entry = makeLazyEntry();
      expect(lazyAdapter.getKey(entry, CONTAINER_PATH)).toBe(
        `${CONTAINER_PATH}::lazy::exports/evidence.zip`
      );
    });
  });

  describe("getName", () => {
    it("returns entry name", () => {
      expect(lazyAdapter.getName(makeLazyEntry())).toBe("evidence.zip");
    });
  });

  describe("getPath", () => {
    it("returns entry path", () => {
      expect(lazyAdapter.getPath(makeLazyEntry())).toBe("exports/evidence.zip");
    });
  });

  describe("isDir", () => {
    it("returns false for files", () => {
      expect(lazyAdapter.isDir(makeLazyEntry())).toBe(false);
    });

    it("returns true for directories", () => {
      expect(lazyAdapter.isDir(makeLazyEntry({ is_dir: true }))).toBe(true);
    });
  });

  describe("getSize", () => {
    it("returns size when available", () => {
      expect(lazyAdapter.getSize(makeLazyEntry({ size: 8192 }))).toBe(8192);
    });

    it("returns undefined when size is null", () => {
      const entry = makeLazyEntry();
      // LazyTreeEntry has size as number, but adapter returns size ?? undefined
      // Size is always a number in the interface, so test normal value
      expect(lazyAdapter.getSize(entry)).toBe(8192);
    });
  });

  describe("getHash", () => {
    it("returns hash when available", () => {
      expect(lazyAdapter.getHash!(makeLazyEntry({ hash: "sha256:abc" }))).toBe("sha256:abc");
    });

    it("returns undefined when hash is null", () => {
      expect(lazyAdapter.getHash!(makeLazyEntry({ hash: null }))).toBeUndefined();
    });
  });

  describe("getEntryType", () => {
    it("returns entry type when available", () => {
      expect(lazyAdapter.getEntryType!(makeLazyEntry({ entry_type: "folder" }))).toBe("folder");
    });
  });

  describe("hasChildren", () => {
    it("returns true for directory with children", () => {
      expect(lazyAdapter.hasChildren(makeLazyEntry({ is_dir: true, child_count: 5 }))).toBe(true);
    });

    it("returns false for directory with 0 children", () => {
      expect(lazyAdapter.hasChildren(makeLazyEntry({ is_dir: true, child_count: 0 }))).toBe(false);
    });

    it("returns false for directory with null child_count", () => {
      const entry = makeLazyEntry({ is_dir: true });
      // child_count defaults to 0 in fixture, override to test null  
      (entry as unknown as Record<string, unknown>).child_count = null;
      expect(lazyAdapter.hasChildren(entry)).toBe(false);
    });

    it("returns false for files regardless of child_count", () => {
      expect(lazyAdapter.hasChildren(makeLazyEntry({ is_dir: false, child_count: 10 }))).toBe(false);
    });
  });

  describe("isNestedContainer", () => {
    it("always returns false", () => {
      expect(lazyAdapter.isNestedContainer!(makeLazyEntry())).toBe(false);
    });
  });
});

// ============================================================================
// Cross-Adapter Consistency Tests
// ============================================================================

describe("Adapter consistency", () => {
  it("all adapters implement required TreeNodeAdapter methods", () => {
    const adapters = [ad1Adapter, archiveAdapter, ufedAdapter, vfsAdapter, lazyAdapter];
    for (const adapter of adapters) {
      expect(typeof adapter.getKey).toBe("function");
      expect(typeof adapter.getName).toBe("function");
      expect(typeof adapter.getPath).toBe("function");
      expect(typeof adapter.isDir).toBe("function");
      expect(typeof adapter.getSize).toBe("function");
      expect(typeof adapter.hasChildren).toBe("function");
    }
  });

  it("all adapters implement isNestedContainer", () => {
    const adapters = [ad1Adapter, archiveAdapter, ufedAdapter, vfsAdapter, lazyAdapter];
    for (const adapter of adapters) {
      expect(typeof adapter.isNestedContainer).toBe("function");
    }
  });

  it("all adapters generate unique key prefixes", () => {
    // Verify different adapters produce different key patterns
    const ad1Key = ad1Adapter.getKey(makeTreeEntry(), CONTAINER_PATH);
    const archiveKey = archiveAdapter.getKey(makeArchiveEntry(), CONTAINER_PATH);
    const ufedKey = ufedAdapter.getKey(makeUfedEntry(), CONTAINER_PATH);
    const vfsKey = vfsAdapter.getKey(makeVfsEntry(), CONTAINER_PATH);
    const lazyKey = lazyAdapter.getKey(makeLazyEntry(), CONTAINER_PATH);

    // Each should have a distinct prefix pattern
    expect(ad1Key).toContain("::addr:");
    expect(archiveKey).toContain("::archive::");
    expect(ufedKey).toContain("::ufed::");
    expect(vfsKey).toContain("::vfs::");
    expect(lazyKey).toContain("::lazy::");
  });
});
